//! Tests mirroring Rocksmith2014.DLCProject.Tests/DLCKeyTests.fs

use rocksmith2014_dlcproject::dlc_key;

#[test]
fn dlc_key_can_be_created() {
    let key = dlc_key::create("creator", "Artist", "Title");
    assert!(!key.is_empty(), "a key was created");
}

#[test]
fn created_dlc_key_has_minimum_length() {
    let key = dlc_key::create("", "", "");
    assert!(
        key.len() >= dlc_key::MINIMUM_LENGTH,
        "the key has at least {} characters",
        dlc_key::MINIMUM_LENGTH
    );
}
