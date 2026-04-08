//! Tests for time-code utility functions.
//!
//! Mirrors `UtilsTest.cs` in Rocksmith2014.NET.
//!
//! Note: The .NET implementation uses `float` (32-bit) arithmetic; our
//! implementation uses `f64`. Results are identical for all normal
//! Rocksmith time values; edge cases with >7 significant digits may differ.

use rocksmith2014_xml::utils::{parse_binary, time_code_from_float_string, time_code_to_string};

// ---------------------------------------------------------------------------
// time_code_to_string
// ---------------------------------------------------------------------------

#[test]
fn time_code_to_string_converts_correctly() {
    assert_eq!(time_code_to_string(0), "0.000");
    assert_eq!(time_code_to_string(18), "0.018");
    assert_eq!(time_code_to_string(235), "0.235");
    assert_eq!(time_code_to_string(1000), "1.000");
    assert_eq!(time_code_to_string(1234), "1.234");
    assert_eq!(time_code_to_string(20500), "20.500");
    assert_eq!(time_code_to_string(989999), "989.999");
    assert_eq!(time_code_to_string(987456123), "987456.123");
}

// ---------------------------------------------------------------------------
// time_code_from_float_string
// ---------------------------------------------------------------------------

#[test]
fn time_code_from_float_string_parses_correctly() {
    assert_eq!(time_code_from_float_string("0.000"), 0);
    assert_eq!(time_code_from_float_string("0.018"), 18);
    assert_eq!(time_code_from_float_string("0.235"), 235);
    assert_eq!(time_code_from_float_string("1.000"), 1000);
    assert_eq!(time_code_from_float_string("1.234"), 1234);
    assert_eq!(time_code_from_float_string("20.500"), 20500);
    assert_eq!(time_code_from_float_string("989.999"), 989999);
    assert_eq!(time_code_from_float_string("1"), 1000);
    assert_eq!(time_code_from_float_string("8.7"), 8700);
    assert_eq!(time_code_from_float_string("6.66"), 6660);
    assert_eq!(time_code_from_float_string("254.112"), 254112);
}

// ---------------------------------------------------------------------------
// parse_binary
// ---------------------------------------------------------------------------

#[test]
fn parse_binary_parses_correctly() {
    assert_eq!(parse_binary("0"), 0);
    assert_eq!(parse_binary("1"), 1);
    assert_eq!(parse_binary("2"), 1);
    assert_eq!(parse_binary("9"), 1);
}
