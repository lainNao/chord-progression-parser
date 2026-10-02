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

    // serde-wasm-bindgen's byte-buffer path also visits ordinary arrays, but unlike
    // deserialize_seq it rejects primitives without calling Reflect.get. Caught
    // exceptions in that lookup currently leak wasm-bindgen reference slots.
    // This visitor only accepts sequences: actual byte buffers are still rejected.
    // Serde's buffered enum content also supports this path when value precedes type.
    deserializer.deserialize_byte_buf(ArrayVisitor(PhantomData))
}
