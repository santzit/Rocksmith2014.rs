//! Utility functions mirroring the .NET `Utils` class in Rocksmith2014.XML.
//!
//! Provides time-code conversion helpers used when reading and writing
//! Rocksmith XML files, as well as `parse_binary` for boolean attributes.

/// Converts a time code in milliseconds to a 3-decimal-place float string.
///
/// Mirrors `Utils.TimeCodeToString` in the .NET reference implementation.
///
/// # Examples
/// ```
/// use rocksmith2014_xml::utils::time_code_to_string;
/// assert_eq!(time_code_to_string(1234), "1.234");
/// assert_eq!(time_code_to_string(0),    "0.000");
/// ```
pub fn time_code_to_string(ms: i32) -> String {
    crate::parser::time_to_str(ms)
}

/// Parses a float string representing seconds into a time code in milliseconds.
///
/// Mirrors `Utils.TimeCodeFromFloatString` in the .NET reference implementation.
/// Uses `f64` arithmetic (the .NET implementation uses `float`/`f32`; results
/// may differ on edge cases with more than 7 significant digits).
///
/// # Examples
/// ```
/// use rocksmith2014_xml::utils::time_code_from_float_string;
/// assert_eq!(time_code_from_float_string("1.234"), 1234);
/// assert_eq!(time_code_from_float_string("0.000"), 0);
/// ```
pub fn time_code_from_float_string(s: &str) -> i32 {
    crate::parser::time_from_str(s)
}

/// Parses a binary (boolean) attribute value: returns `0` for `"0"`, `1` for everything else.
///
/// Mirrors `Utils.ParseBinary` in the .NET reference implementation.
///
/// # Examples
/// ```
/// use rocksmith2014_xml::utils::parse_binary;
/// assert_eq!(parse_binary("0"), 0);
/// assert_eq!(parse_binary("1"), 1);
/// assert_eq!(parse_binary("2"), 1);
/// ```
pub fn parse_binary(s: &str) -> u8 {
    if s == "0" {
        0
    } else {
        1
    }
}
