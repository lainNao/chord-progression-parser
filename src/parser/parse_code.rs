use regex::Regex;

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

        return Ok(Chord {
            chord,
            sharp_flat,
            chord_type,
            modifier,
        });
    }

    Err("Invalid chord format.")
}

fn main() {
    match parse_chord("Am7") {
        Ok(chord) => println!("{:?}", chord),
        Err(e) => println!("Error: {}", e),
    }
}
