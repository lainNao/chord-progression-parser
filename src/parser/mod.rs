mod types;
mod util;

use crate::errors;
use crate::tokenizer::types::Token;

use types::{Ast, Chord, ChordInfo, ChordInfoMeta, Key, Section, SectionMeta};

pub fn parse(tokens: &[Token]) -> Result<Ast, String> {
    let mut sections: Vec<Section> = Vec::new();
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            Token::SectionMetaInfoStart => {
                let is_new_section = 
                    // no section
                    sections.is_empty() ||
                    // last section's chord_blocks is not empty
                    !sections.last().unwrap().chord_blocks.is_empty();
                
                // if is_new_section, initialize new section
                if is_new_section {
                    sections.push(Section {
                        meta_infos: Vec::new(),
                        chord_blocks: Vec::new(),
                    });
                }

                // if next token is not Token::SectionMetaInfoKey, return error
                let section_meta_info_key = match tokens.next() {
                    Some(Token::SectionMetaInfoKey(value)) => value,
                    _ => return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string()),
                };

                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string())
                }

                // if next token is not Token::SectionMetaInfoValue, return error
                let section_meta_info_value = match tokens.next() {
                    Some(Token::SectionMetaInfoValue(value)) => value,
                    _ => return Err(errors::SECTION_META_INFO_VALUE_SHOULD_NOT_BE_EMPTY.to_string()),
                };

                // add section meta info to last section
                match section_meta_info_key.as_str() {
                    "section" => {
                        sections.last_mut().unwrap().meta_infos.push(SectionMeta::Section {
                            value: section_meta_info_value.clone(),
                        });
                    }
                    "repeat" => {
                        // if section_meta_info_value cannot parse as u32, return error
                        if section_meta_info_value.parse::<u32>().is_err() {
                            return Err(errors::SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER.to_string());
                        }

                        sections.last_mut().unwrap().meta_infos.push(SectionMeta::Repeat {
                            value: section_meta_info_value.parse::<u32>().unwrap(),
                        });
                    }
                    _ => {
                        return Err(errors::SECTION_META_INFO_KEY_IS_INVALID.to_string());
                    }
                }
            
                match tokens.next() {
                    Some(Token::LineBreak) => {}
                    _ => {
                        return Err(errors::SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER.to_string());
                    }
                }
            },
            Token::MetaInfoStart => {//(
                
                // if next token is not Token::MetaInfoKey, return error
                let meta_info_key = match tokens.next() {
                    Some(Token::MetaInfoKey(value)) => value,
                    _ => return Err(errors::META_INFO_KEY_SHOULD_NOT_BE_EMPTY.to_string()),
                };
        
                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => return Err(errors::META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string())
                }
        
                // if next token is not Token::MetaInfoValue, return error
                let meta_info_value = match tokens.next() {
                    Some(Token::MetaInfoValue(value)) => value,
                    _ => return Err(errors::META_INFO_VALUE_SHOULD_NOT_BE_EMPTY.to_string()),
                };
        
                // add meta info to last chord block
                match meta_info_key.as_str() {
                    "key" => {
                        // if section_meta_info_value cannot parse as u32, return error
                        if meta_info_value.parse::<u32>().is_err() {
                            return Err(errors::SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER.to_string());
                        }

                        // add ChordInfoMeta to last ChordInfo
                        let last_chord_block = sections.last_mut().unwrap().chord_blocks.last_mut().unwrap();

                        last_chord_block.last_mut().unwrap().meta_infos.push(ChordInfoMeta::Key {
                            value: Key::C_M, //TODO コードはmatchなどで判定する
                        });
                    }
                    _ => {
                        return Err(errors::META_INFO_KEY_IS_INVALID.to_string());
                    }
                }
            
                // match tokens.next() {
                //     Some(Token::LineBreak) => {}
                //     _ => {
                //         return Err(errors::SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER.to_string());
                //     }
                // }    
            }       
            // Token::MetaInfoKey(String) => {}
            // Token::MetaInfoValue(String) => {}
            // Token::MetaInfoEnd => {} //)
            Token::ChordBlockSeparator => {} // |
            Token::Chord(String) => {}
            Token::Equal => {}
            Token::Comma => {}
            Token::LineBreak => {}
            _ => {}
        }
    }

    Ok(sections)
}
