//! Mirrors BinaryHelpersTests.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::{write_zero_terminated_utf8_string, Error, Sng};

/// Mirrors: "UTF8 string always includes null terminator"
#[test]
fn utf8_string_always_includes_null_terminator() {
    let length = 48;
    let input = "A".repeat(length);
    let mut buf = Vec::new();
    write_zero_terminated_utf8_string(&mut buf, length, &input).unwrap();
    assert_eq!(buf.len(), length);
    assert_eq!(buf[length - 1], 0u8, "Last byte is zero");
}

#[test]
fn test_negative_array_count_returns_error() {
    let crafted: Vec<u8> = (-1i32).to_le_bytes().to_vec();
    let result = Sng::read(&crafted);
    match result {
        Err(Error::InvalidArrayCount(n)) => assert_eq!(n, -1),
        other => panic!("expected InvalidArrayCount(-1), got {:?}", other),
    }
}
