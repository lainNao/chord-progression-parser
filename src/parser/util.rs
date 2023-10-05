use regex::Regex;

use super::types::{Accidental, Base, Chord, ChordDetailed};

// pub fn parse_chord(input: &str) -> Result<Chord, &'static str> {
// let chord_regexp = format!(
//     r"([A-G])(#|b)?(m|M|aug|dim|add|sus2|sus4)?(2|3|b3|4|b5|-5|5|#5|b6|6|7|b9|9|#9|b11|11|#11|b13|13|#13)?",
//     "#"
// );
// let re = Regex::new(&chord_regexp).unwrap();

// if let Some(captures) = re.captures(input) {
//     let base = captures
//         .get(1)
//         .map_or("".to_string(), |m| m.as_str().to_string());
//     //TODO baseは文字列じゃなくparse_baseの結果をもたせる。以下も似たイメージ

//     let accidental = captures.get(2).map(|m| m.as_str().to_string());
//     let chord_type = captures.get(3).map(|m| m.as_str().to_string());
//     let modifier = captures.get(4).map(|m| m.as_str().to_string());

// return Ok(Chord {
//     plain: input.to_string(),
//     detailed: ChordDetailed {
//         base: parse_base(base),
//         accidental: accidental,
//         chord_type: ChordType::M,
//         extension: None,
//     }, // ,
//        // sharp_flat,
//        // chord_type,
//        // modifier,
// });
// }

// Err("Invalid chord format.")
// }

// fn main() {
//     match parse_chord("Am7") {
//         Ok(chord) => println!("{:?}", chord),
//         Err(e) => println!("Error: {}", e),
//     }
// }

pub fn parse_base(input: &str) -> Result<Base, &'static str> {
    match input {
        "A" => Ok(Base::A),
        "B" => Ok(Base::B),
        "C" => Ok(Base::C),
        "D" => Ok(Base::D),
        "E" => Ok(Base::E),
        "F" => Ok(Base::F),
        "G" => Ok(Base::G),
        _ => Err("Invalid base."),
    }
}

pub fn parse_accidental(input: &str) -> Result<Accidental, &'static str> {
    match input {
        "#" => Ok(Accidental::Sharp),
        "b" => Ok(Accidental::Flat),
        _ => Err("Invalid accidental."),
    }
}

/*
    match parse_chord("Am7") {
        Ok(chord) => println!("{:?}", chord),
        Err(e) => println!("Error: {}", e),
    }
*/
pub fn parse_chord(input: &str) -> Result<Chord, &'static str> {
    let chord_regexp = format!(
        r"([A-G])({}|b)?(m|M|aug|dim|add|sus2|sus4)?(2|3|b3|4|b5|-5|5|#5|b6|6|7|b9|9|#9|b11|11|#11|b13|13|#13)?",
        "#"
    );
    let re = Regex::new(&chord_regexp).unwrap();

    if let Some(captures) = re.captures(input) {
        let chord = captures
            .get(1)
            .map_or("".to_string(), |m| m.as_str().to_string());
        let sharp_flat = captures.get(2).map(|m| m.as_str().to_string());
        let chord_type = captures.get(3).map(|m| m.as_str().to_string());
        let modifier = captures.get(4).map(|m| m.as_str().to_string());

        // return Ok(Chord {
        //     plain,
        //     detailed,
        //     chord,
        //     sharp_flat,
        //     chord_type,
        //     modifier,
        // });
    }

    Err("Invalid chord format.")
}
