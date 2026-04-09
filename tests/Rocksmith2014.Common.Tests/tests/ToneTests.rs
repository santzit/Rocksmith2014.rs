//! Tone XML/JSON import/export tests.
//!
//! Mirrors `ToneTests.fs` in Rocksmith2014.Common.Tests (.NET).
//!
//! NOTE: The `Tone` manifest type and its XML/JSON serialization are not yet
//! implemented in the Rust `rocksmith2014-common` crate. These tests are
//! provided as stubs that will be filled in once the backing implementation
//! exists. They are marked `#[ignore]` so the test suite still compiles and
//! runs without failures.

/// test "Can be imported from XML"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// Expect.equal tone.Key "Test" "Key is correct"
/// ...
#[test]
#[ignore = "Tone XML import not yet implemented"]
fn can_be_imported_from_xml() {
    // TODO: implement once rocksmith2014-common has a Tone/Manifest module
}

/// testTask "Can be exported to XML"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// ...
#[test]
#[ignore = "Tone XML export not yet implemented"]
fn can_be_exported_to_xml() {
    // TODO: implement once rocksmith2014-common has a Tone/Manifest module
}

/// testTask "Can be exported to JSON and imported from JSON"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// ...
#[test]
#[ignore = "Tone JSON round-trip not yet implemented"]
fn can_be_exported_to_json_and_imported_from_json() {
    // TODO: implement once rocksmith2014-common has a Tone/Manifest module
}

/// test "Number of effects can be counted"
///
/// Expect.equal count 6 "Gear list has 6 effects"
#[test]
#[ignore = "Tone effect counting not yet implemented"]
fn number_of_effects_can_be_counted() {
    // TODO: implement once rocksmith2014-common has a Tone/Manifest module
}
