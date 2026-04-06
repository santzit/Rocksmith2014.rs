//! Time-code helpers mirroring `Rocksmith2014.XML.Utils` from Rocksmith2014.NET v3.5.0.

/// Converts a time in **milliseconds** to the `"seconds.mmm"` string format
/// used in Rocksmith 2014 XML files.
///
/// # Examples
/// ```
/// use rocksmith2014::xml::utils::time_code_to_string;
/// assert_eq!(time_code_to_string(0),        "0.000");
/// assert_eq!(time_code_to_string(18),       "0.018");
/// assert_eq!(time_code_to_string(1234),     "1.234");
/// assert_eq!(time_code_to_string(987456123),"987456.123");
/// ```
pub fn time_code_to_string(time_code: i32) -> String {
    let s = time_code.to_string();
    match s.len() {
        1 => format!("0.00{s}"),
        2 => format!("0.0{s}"),
        3 => format!("0.{s}"),
        n => {
            let (secs, ms) = s.split_at(n - 3);
            format!("{secs}.{ms}")
        }
    }
}

/// Parses a `"seconds.fractional"` string into **milliseconds**, truncating to
/// 3 decimal places.  Mirrors `Rocksmith2014.XML.Utils.TimeCodeFromFloatString`.
///
/// # Examples
/// ```
/// use rocksmith2014::xml::utils::time_code_from_float_string;
/// assert_eq!(time_code_from_float_string("0.000"),         0);
/// assert_eq!(time_code_from_float_string("1"),          1000);
/// assert_eq!(time_code_from_float_string("8.7"),        8700);
/// assert_eq!(time_code_from_float_string("18.00599"),  18005);
/// assert_eq!(time_code_from_float_string("9504.11299999"), 9504112);
/// ```
pub fn time_code_from_float_string(input: &str) -> i32 {
    match input.find('.') {
        None => {
            // No decimal separator — treat the whole value as seconds
            input.parse::<i32>().unwrap_or(0) * 1000
        }
        Some(dot_idx) => {
            let int_part = &input[..dot_idx];
            let frac_part = &input[dot_idx + 1..];

            // Take at most 3 decimal digits, right-pad with '0' to 3
            let mut buf = [b'0'; 3];
            let copy = frac_part.len().min(3);
            buf[..copy].copy_from_slice(frac_part[..copy].as_bytes());
            let frac_str = std::str::from_utf8(&buf).unwrap();

            let combined = format!("{int_part}{frac_str}");
            combined.parse::<i32>().unwrap_or(0)
        }
    }
}

/// Parses a `"0"` or `"1"` string into a `u8`, returning `1` for any
/// non-zero character value.  Mirrors `Rocksmith2014.XML.Utils.ParseBinary`.
///
/// # Examples
/// ```
/// use rocksmith2014::xml::utils::parse_binary;
/// assert_eq!(parse_binary("0"), 0);
/// assert_eq!(parse_binary("1"), 1);
/// assert_eq!(parse_binary("9"), 1);
/// ```
pub fn parse_binary(text: &str) -> u8 {
    let c = text.chars().next().unwrap_or('0');
    let n = (c as u8).wrapping_sub(b'0');
    if n >= 1 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------ //
    // TimeCodeToString  (mirrors UtilsTest.TimeCodeToString_ConvertsCorrectly)
    // ------------------------------------------------------------------ //
    #[test]
    fn time_code_to_string_converts_correctly() {
        let cases: &[(i32, &str)] = &[
            (0,         "0.000"),
            (18,        "0.018"),
            (235,       "0.235"),
            (1000,      "1.000"),
            (1234,      "1.234"),
            (20500,     "20.500"),
            (989999,    "989.999"),
            (987456123, "987456.123"),
        ];
        for &(input, expected) in cases {
            assert_eq!(
                time_code_to_string(input), expected,
                "time_code_to_string({input}) should be {expected}"
            );
        }
    }

    // ------------------------------------------------------------------ //
    // TimeCodeFromFloatString  (mirrors UtilsTest.TimeCodeFromFloatString_ParsesCorrectly)
    // ------------------------------------------------------------------ //
    #[test]
    fn time_code_from_float_string_parses_correctly() {
        let cases: &[(&str, i32)] = &[
            ("0.000",         0),
            ("0.018",        18),
            ("0.235",       235),
            ("1.000",      1000),
            ("1.234",      1234),
            ("20.500",    20500),
            ("989.999",  989999),
            ("1",          1000),
            ("8.7",        8700),
            ("6.66",       6660),
            ("18.00599",  18005),
            ("254.112",  254112),
            ("9504.11299999", 9504112),
        ];
        for &(input, expected) in cases {
            assert_eq!(
                time_code_from_float_string(input), expected,
                "time_code_from_float_string({input:?}) should be {expected}"
            );
        }
    }

    // ------------------------------------------------------------------ //
    // ParseBinary  (mirrors UtilsTest.ParseBinary_ParsesCorrectly)
    // ------------------------------------------------------------------ //
    #[test]
    fn parse_binary_parses_correctly() {
        assert_eq!(parse_binary("0"), 0);
        assert_eq!(parse_binary("1"), 1);
        assert_eq!(parse_binary("2"), 1);
        assert_eq!(parse_binary("9"), 1);
    }
}
