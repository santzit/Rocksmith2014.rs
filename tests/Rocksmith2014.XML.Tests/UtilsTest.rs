//! Tests for time-code utility functions.
//!
//! Mirrors `UtilsTest.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::utils::{parse_binary, time_code_from_float_string, time_code_to_string};

// ---------------------------------------------------------------------------
// TimeCodeToString — 8 cases ([InlineData] in .NET)
// ---------------------------------------------------------------------------

#[test]
fn time_code_to_string_0() {
    assert_eq!(time_code_to_string(0), "0.000");
}

#[test]
fn time_code_to_string_18() {
    assert_eq!(time_code_to_string(18), "0.018");
}

#[test]
fn time_code_to_string_235() {
    assert_eq!(time_code_to_string(235), "0.235");
}

#[test]
fn time_code_to_string_1000() {
    assert_eq!(time_code_to_string(1000), "1.000");
}

#[test]
fn time_code_to_string_1234() {
    assert_eq!(time_code_to_string(1234), "1.234");
}

#[test]
fn time_code_to_string_20500() {
    assert_eq!(time_code_to_string(20500), "20.500");
}

#[test]
fn time_code_to_string_989999() {
    assert_eq!(time_code_to_string(989999), "989.999");
}

#[test]
fn time_code_to_string_987456123() {
    assert_eq!(time_code_to_string(987456123), "987456.123");
}

// ---------------------------------------------------------------------------
// TimeCodeFromFloatString — 13 cases ([InlineData] in .NET)
// ---------------------------------------------------------------------------

#[test]
fn time_code_from_float_string_0_000() {
    assert_eq!(time_code_from_float_string("0.000"), 0);
}

#[test]
fn time_code_from_float_string_0_018() {
    assert_eq!(time_code_from_float_string("0.018"), 18);
}

#[test]
fn time_code_from_float_string_0_235() {
    assert_eq!(time_code_from_float_string("0.235"), 235);
}

#[test]
fn time_code_from_float_string_1_000() {
    assert_eq!(time_code_from_float_string("1.000"), 1000);
}

#[test]
fn time_code_from_float_string_1_234() {
    assert_eq!(time_code_from_float_string("1.234"), 1234);
}

#[test]
fn time_code_from_float_string_20_500() {
    assert_eq!(time_code_from_float_string("20.500"), 20500);
}

#[test]
fn time_code_from_float_string_989_999() {
    assert_eq!(time_code_from_float_string("989.999"), 989999);
}

#[test]
fn time_code_from_float_string_1_integer() {
    assert_eq!(time_code_from_float_string("1"), 1000);
}

#[test]
fn time_code_from_float_string_8_7() {
    assert_eq!(time_code_from_float_string("8.7"), 8700);
}

#[test]
fn time_code_from_float_string_6_66() {
    assert_eq!(time_code_from_float_string("6.66"), 6660);
}

#[test]
fn time_code_from_float_string_18_00599() {
    assert_eq!(time_code_from_float_string("18.00599"), 18005);
}

#[test]
fn time_code_from_float_string_254_112() {
    assert_eq!(time_code_from_float_string("254.112"), 254112);
}

#[test]
fn time_code_from_float_string_9504_112() {
    assert_eq!(time_code_from_float_string("9504.11299999"), 9504112);
}

// ---------------------------------------------------------------------------
// ParseBinary — 4 cases ([InlineData] in .NET)
// ---------------------------------------------------------------------------

#[test]
fn parse_binary_0() {
    assert_eq!(parse_binary("0"), 0);
}

#[test]
fn parse_binary_1() {
    assert_eq!(parse_binary("1"), 1);
}

#[test]
fn parse_binary_2() {
    assert_eq!(parse_binary("2"), 1);
}

#[test]
fn parse_binary_9() {
    assert_eq!(parse_binary("9"), 1);
}
