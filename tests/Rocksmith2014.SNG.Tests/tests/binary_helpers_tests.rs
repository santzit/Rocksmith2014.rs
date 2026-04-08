//! Mirrors BinaryHelpersTests.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::{Error, Sng};

#[test]
fn test_negative_array_count_returns_error() {
    let crafted: Vec<u8> = (-1i32).to_le_bytes().to_vec();
    let result = Sng::read(&crafted);
    match result {
        Err(Error::InvalidArrayCount(n)) => assert_eq!(n, -1),
        other => panic!("expected InvalidArrayCount(-1), got {:?}", other),
    }
}
