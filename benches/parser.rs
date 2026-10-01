use std::{
    fmt::Write,
    hint::black_box,
    time::{Duration, Instant},
};

use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, ChordDetailed,
};

/** Measures a batch including destruction of returned ASTs and diagnostic vectors. */
fn timed_batch<T>(iterations: u64, operation: &mut impl FnMut() -> T) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(operation());
    }
    started.elapsed()
}

/** Calibrates a deterministic workload and prints seven samples as one CSV summary. */
fn measure<T>(name: &str, input_bytes: usize, mut operation: impl FnMut() -> T) {
    let minimum_sample = Duration::from_millis(100);
    let mut iterations = 1;
    while timed_batch(iterations, &mut operation) < minimum_sample {
        iterations *= 2;
    }
    let mut samples = [0.0; 7];
    for sample in &mut samples {
        *sample =
            timed_batch(iterations, &mut operation).as_secs_f64() * 1_000_000.0 / iterations as f64;
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{name},{input_bytes},{:.3},{:.3},{:.3}",
        samples[3], samples[0], samples[6]
    );
}

/** Benchmarks normal parsing, warning-heavy input, error recovery, Unicode, and formatting. */
fn main() {
    let short = "[key=C]C(M9)-Am(7)-Dm(7)-G(7)/B";
    let progression = "C(9,9)-Am(7)-Dm(7)-G/B\n".repeat(100);
    let duplicates = format!("C({})", vec!["9"; 1_000].join(","));
    let errors = vec!["H"; 1_000].join("-");
    let unicode = format!("@section=日本語😀\r\n{}", "C/(😀あ)-Dm(7)\r\n".repeat(100));

    // Validate fixtures outside the timed loops, so an accidentally rejected input
    // cannot appear to be a parser performance improvement.
    parse_chord_progression_string(short).expect("short fixture must parse");
    let progression_ast =
        parse_chord_progression_string(&progression).expect("progression fixture must parse");
    assert_eq!(
        parse_chord_progression_string_with_warnings(&progression)
            .warnings
            .len(),
        100
    );
    let duplicate_report = parse_chord_progression_string_with_warnings(&duplicates);
    assert_eq!(duplicate_report.warnings.len(), 999);
    let duplicate_ast = duplicate_report
        .result
        .expect("duplicate extensions are valid");
    let diagnostics = parse_chord_progression_string(&errors).unwrap_err();
    assert_eq!(diagnostics.len(), 1_000);
    assert_eq!(diagnostics[0].error.to_string(), "CHO-1: BS-1: H");
    parse_chord_progression_string(&unicode).expect("Unicode fixture must parse");

    println!("case,input_bytes,median_us,min_us,max_us");
    measure("parse_short", short.len(), || {
        parse_chord_progression_string_with_warnings(black_box(short))
    });
    measure("parse_progression", progression.len(), || {
        parse_chord_progression_string_with_warnings(black_box(&progression))
    });
    measure("parse_duplicates", duplicates.len(), || {
        parse_chord_progression_string_with_warnings(black_box(&duplicates))
    });
    measure(
        "parse_duplicates_without_warnings",
        duplicates.len(),
        || parse_chord_progression_string(black_box(&duplicates)),
    );
    measure("parse_errors", errors.len(), || {
        parse_chord_progression_string_with_warnings(black_box(&errors))
    });
    measure("parse_unicode", unicode.len(), || {
        parse_chord_progression_string_with_warnings(black_box(&unicode))
    });
    measure("format_progression", progression.len(), || {
        format_chord_progression(black_box(&progression_ast)).expect("valid AST must format")
    });
    measure("format_duplicates", duplicates.len(), || {
        format_chord_progression(black_box(&duplicate_ast)).expect("valid AST must format")
    });
    measure("format_diagnostics", errors.len(), || {
        let mut output = String::new();
        for diagnostic in black_box(&diagnostics) {
            writeln!(output, "{}", diagnostic.error).expect("writing to a String cannot fail");
        }
        output
    });
    let chord = "C#dim(1,b3,b5,7,9,#11,13)";
    chord
        .parse::<ChordDetailed>()
        .expect("chord fixture must parse");
    measure("parse_chord", chord.len(), || {
        black_box(chord).parse::<ChordDetailed>()
    });
}
