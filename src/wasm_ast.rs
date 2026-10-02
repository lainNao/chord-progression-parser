use std::{fmt, marker::PhantomData};

use serde::{
    de::{value::SeqAccessDeserializer, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

/** Reads AST arrays without invoking JavaScript iterable lookup on malformed values. */
pub(crate) fn deserialize_array<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ArrayVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for ArrayVisitor<T> {
        type Value = Vec<T>;

        /** Describes the public AST field's expected representation. */
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array")
        }

        /** Keeps Vec's usual allocation and element deserialization. */
        fn visit_seq<A: SeqAccess<'de>>(self, sequence: A) -> Result<Self::Value, A::Error> {
            Vec::deserialize(SeqAccessDeserializer::new(sequence))
        }
    }

    if !deserializer.is_human_readable() {
        // Binary Rust serializers retain Vec's sequence representation.
        return Vec::deserialize(deserializer);
    }

    // serde-wasm-bindgen checks primitives before reaching the array branch in
    // deserialize_any. Its deserialize_seq instead performs iterable lookup on
    // malformed primitives, whose caught exceptions leak reference slots.
    // Do not use deserialize_byte_buf: text formats such as YAML can support
    // ordinary sequences while rejecting byte-buffer deserialization entirely.
    // Only sequences are accepted here; byte buffers and other values are rejected.
    deserializer.deserialize_any(ArrayVisitor(PhantomData))
}

#[cfg(test)]
mod tests {
    use serde::{de::Error, de::Visitor, Deserializer};

    use super::deserialize_array;

    /** Models sequence formats that lack byte-buffer support, as YAML does. */
    struct SequenceDeserializer<'de> {
        source: &'de str,
        human_readable: bool,
    }

    impl<'de> Deserializer<'de> for SequenceDeserializer<'de> {
        type Error = serde_json::Error;

        /** Self-describing text formats can inspect the ordinary sequence. */
        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            if !self.human_readable {
                return Err(Self::Error::custom("format requires a sequence hint"));
            }
            serde_json::Deserializer::from_str(self.source).deserialize_any(visitor)
        }

        /** Binary formats retain Vec's explicit sequence hint. */
        fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            serde_json::Deserializer::from_str(self.source).deserialize_seq(visitor)
        }

        /** Supporting arrays does not require a format to support byte buffers. */
        fn deserialize_byte_buf<V: Visitor<'de>>(
            self,
            _visitor: V,
        ) -> Result<V::Value, Self::Error> {
            Err(Self::Error::custom("format does not support byte buffers"))
        }

        /** Keeps the binary and text cases independent of their JSON test fixture. */
        fn is_human_readable(&self) -> bool {
            self.human_readable
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            option unit unit_struct newtype_struct tuple tuple_struct map struct
            enum identifier ignored_any
        }
    }

    /** Guards text sequence compatibility without requiring a byte-buffer API. */
    #[test]
    fn reads_text_sequences_without_byte_buffer_support() {
        let values: Vec<u8> = deserialize_array(SequenceDeserializer {
            source: "[1,9,9]",
            human_readable: true,
        })
        .expect("a text sequence must not require byte-buffer deserialization");
        assert_eq!(values, vec![1, 9, 9]);
    }

    /** Preserves formats that need type hints instead of deserialize_any. */
    #[test]
    fn reads_binary_sequences_without_self_describing_support() {
        let values: Vec<u8> = deserialize_array(SequenceDeserializer {
            source: "[1,9,9]",
            human_readable: false,
        })
        .expect("a binary sequence must retain Vec's sequence hint");
        assert_eq!(values, vec![1, 9, 9]);
    }

    /** Rejects non-array text values through the same human-readable path. */
    #[test]
    fn rejects_non_sequence_text_values() {
        for source in ["null", "true", "1", "\"bad\"", "{}"] {
            let values: Result<Vec<u8>, _> = deserialize_array(SequenceDeserializer {
                source,
                human_readable: true,
            });
            assert!(values.is_err(), "unexpectedly accepted {source}");
        }
    }
}
