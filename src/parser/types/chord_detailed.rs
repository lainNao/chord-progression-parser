use std::string;

use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};
use strum::VariantNames;

#[derive(Debug, PartialEq, Clone)]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extension: Option<Extension>,
    pub additional_extension: Option<Extension>,
}

fn try_remove_prefix(input: &str, prefix: &str) -> String {
    match input.strip_prefix(prefix) {
        Some(stripped) => String::from(stripped),
        None => String::from(input),
    }
}

impl ChordDetailed {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let base = match s.chars().next() {
            Some('A') => Base::A,
            Some('B') => Base::B,
            Some('C') => Base::C,
            Some('D') => Base::D,
            Some('E') => Base::E,
            Some('F') => Base::F,
            Some('G') => Base::G,
            _ => return Err("Invalid base.".to_string()),
        };

        let mut idx = 1; // Start after the base note

        let accidental = match s.chars().nth(idx) {
            Some('#') => {
                idx += 1;
                Some(Accidental::Sharp)
            }
            Some('b') => {
                idx += 1;
                Some(Accidental::Flat)
            }
            // No accidental
            _ => None,
        };

        let chord_str_without_base = &s[idx..];

        let chord_type = if chord_str_without_base.starts_with('m') {
            ChordType::Minor
        } else if chord_str_without_base.starts_with('M') {
            ChordType::Major
        } else if chord_str_without_base.starts_with("aug") {
            ChordType::Augmented
        } else if chord_str_without_base.starts_with("dim") {
            ChordType::Diminished
        } else {
            ChordType::Major
        };

        let extensions_str = try_remove_prefix(
            chord_str_without_base,
            match chord_type {
                ChordType::Minor => "m",
                ChordType::Major => "M",
                ChordType::Augmented => "aug",
                ChordType::Diminished => "dim",
            },
        );

        if extensions_str.as_str().eq("") {
            return Ok(ChordDetailed {
                base,
                accidental,
                chord_type,
                extension: None,
                additional_extension: None,
            });
        }

        let mut sorted_extensions = (*Extension::VARIANTS.clone()).to_vec();
        sorted_extensions.sort_by_key(|b| std::cmp::Reverse(b.len()));
        let first_extension_str_result = sorted_extensions
            .iter()
            .find(|e| extensions_str.starts_with(*e));
        if first_extension_str_result.is_none() {
            return Ok(ChordDetailed {
                base,
                accidental,
                chord_type,
                extension: None,
                additional_extension: None,
            });
        }

        // TODO 繰り返してる＆重複定義になってるのでリファクタ
        let first_extension_str = first_extension_str_result.unwrap();
        let first_extension: Option<Extension> = match *first_extension_str {
            "2" => Some(Extension::Two),
            "3" => Some(Extension::Three),
            "b3" => Some(Extension::FlatThree),
            "4" => Some(Extension::Four),
            "b5" => Some(Extension::FlatFive),
            "-5" => Some(Extension::FlatFive),
            "5" => Some(Extension::Five),
            "#5" => Some(Extension::SharpFive),
            "b6" => Some(Extension::FlatSix),
            "6" => Some(Extension::Six),
            "7" => Some(Extension::Seven),
            "b9" => Some(Extension::FlatNine),
            "9" => Some(Extension::Nine),
            "#9" => Some(Extension::SharpNine),
            "b11" => Some(Extension::FlatEleven),
            "11" => Some(Extension::Eleven),
            "#11" => Some(Extension::SharpEleven),
            "b13" => Some(Extension::FlatThirteen),
            "13" => Some(Extension::Thirteen),
            "#13" => Some(Extension::SharpThirteen),
            "add9" => Some(Extension::Add9),
            "add11" => Some(Extension::Add11),
            "add13" => Some(Extension::Add13),
            "sus2" => Some(Extension::Sus2),
            "sus4" => Some(Extension::Sus4),
            "o" => Some(Extension::HalfDiminish),
            _ => None,
        };

        let additional_extension_str = extensions_str.strip_prefix(first_extension_str);
        let additional_extension: Option<Extension> = match additional_extension_str {
            Some("2") => Some(Extension::Two),
            Some("3") => Some(Extension::Three),
            Some("b3") => Some(Extension::FlatThree),
            Some("4") => Some(Extension::Four),
            Some("b5") => Some(Extension::FlatFive),
            Some("-5") => Some(Extension::FlatFive),
            Some("5") => Some(Extension::Five),
            Some("#5") => Some(Extension::SharpFive),
            Some("b6") => Some(Extension::FlatSix),
            Some("6") => Some(Extension::Six),
            Some("7") => Some(Extension::Seven),
            Some("b9") => Some(Extension::FlatNine),
            Some("9") => Some(Extension::Nine),
            Some("#9") => Some(Extension::SharpNine),
            Some("b11") => Some(Extension::FlatEleven),
            Some("11") => Some(Extension::Eleven),
            Some("#11") => Some(Extension::SharpEleven),
            Some("b13") => Some(Extension::FlatThirteen),
            Some("13") => Some(Extension::Thirteen),
            Some("#13") => Some(Extension::SharpThirteen),
            Some("add9") => Some(Extension::Add9),
            Some("add11") => Some(Extension::Add11),
            Some("add13") => Some(Extension::Add13),
            Some("sus2") => Some(Extension::Sus2),
            Some("sus4") => Some(Extension::Sus4),
            Some("o") => Some(Extension::HalfDiminish),
            _ => None,
        };

        Ok(ChordDetailed {
            base,
            accidental,
            chord_type,
            extension: first_extension,
            additional_extension,
        })
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod success {

        mod complex_chords {
            use std::str::FromStr;

            use strum::VariantNames;

            use crate::parser::types::{
                accidental::Accidental, base::Base, chord_detailed::ChordDetailed,
                chord_type::ChordType, extension::Extension,
            };

            #[test]
            fn all_extension() {
                Extension::VARIANTS.iter().for_each(|extension_str| {
                    let mut chord_str = String::from("C");
                    chord_str.push_str(extension_str);
                    let extension = Extension::from_str(*extension_str);

                    // FIXME: 「Cb9」が「Cbの9」なのか「Cのb9」なのか分からず、強制的に前者扱いになってしまい、failしてしまう…どうしようかな。
                    // これって世のコード表記自体の脆弱性なのでは？かっこをつけてるのかそういうのは…
                    // 対策案としてはテンション部分は全部かっこで囲うルールにするとか…うーんでもそしたら「C(7)」とかになっちゃうけども…いいかな？うーん。C(5)
                    //   もしそうするなら、コードメタ情報は()で囲うのでなく[]で囲うように修正したほうがいいな。というかそれはもうやっちゃってもいいはず。紛らわしいので

                    // TODO: ExtensionにM7、M9、M11、M13を忘れてたわ。追加
                    // これも「CM9」だと「CMの9th」なのか「CのM9」なのか区別つかなくなる問題あるな。
                    //   やっぱり前者を「CM(9)」、後者を「C（M9)」のようにかき分けるのが正義か…？なんかそう思えてきた
                    //   というかそういう仕様で提案したら？その方がExtensionをいくらでもつけれるよね？（C(-5,9,13)とか）
                    //   あと知恵袋に「CM9」の問題を聞いてみてもサンプリングにいいかもね

                    println!("chord_str: {}", chord_str);
                    println!("extension: {:?}", extension);

                    assert_eq!(
                        ChordDetailed::from_str(&chord_str).unwrap(),
                        ChordDetailed {
                            base: Base::C,
                            accidental: None,
                            chord_type: ChordType::Major,
                            extension: Some(extension.unwrap()),
                            additional_extension: None,
                        }
                    );
                });
            }

            #[test]
            fn half_diminish_raw() {
                assert_eq!(
                    ChordDetailed::from_str("C#m7-5").unwrap(),
                    ChordDetailed {
                        base: Base::C,
                        accidental: Some(Accidental::Sharp),
                        chord_type: ChordType::Minor,
                        extension: Some(Extension::Seven),
                        additional_extension: Some(Extension::FlatFive),
                    }
                );
            }

            #[test]
            fn half_diminish_symbol() {
                assert_eq!(
                    ChordDetailed::from_str("Co11").unwrap(),
                    ChordDetailed {
                        base: Base::C,
                        accidental: None,
                        chord_type: ChordType::Major,
                        extension: Some(Extension::HalfDiminish),
                        additional_extension: Some(Extension::Eleven),
                    }
                );
            }

            #[test]
            fn nine_add13() {
                assert_eq!(
                    ChordDetailed::from_str("C#m9add13").unwrap(),
                    ChordDetailed {
                        base: Base::C,
                        accidental: Some(Accidental::Sharp),
                        chord_type: ChordType::Minor,
                        extension: Some(Extension::Nine),
                        additional_extension: Some(Extension::Add13),
                    }
                );
            }

            #[test]
            fn nine_flat13() {
                assert_eq!(
                    ChordDetailed::from_str("C#m9b13").unwrap(),
                    ChordDetailed {
                        base: Base::C,
                        accidental: Some(Accidental::Sharp),
                        chord_type: ChordType::Minor,
                        extension: Some(Extension::Nine),
                        additional_extension: Some(Extension::FlatThirteen),
                    }
                );
            }
        }
    }
}
