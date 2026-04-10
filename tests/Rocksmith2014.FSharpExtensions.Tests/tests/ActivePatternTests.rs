//! Tests mirroring Rocksmith2014.FSharpExtensions.Tests/ActivePatternTests.fs
//!
//! In F#, `Contains`, `EndsWith`, and `StartsWith` are partial active patterns
//! (case-insensitive matching). In Rust these map to the `*_ignore_case` helpers
//! in `string_ext`.

use rocksmith2014_fsharp_extensions::string_ext::{
    contains_ignore_case, ends_with_ignore_case, starts_with_ignore_case,
};

const TEST_STRING: &str = "Some STRING";

// --- Contains ---

#[test]
fn contains_ignores_case() {
    assert!(contains_ignore_case("string", TEST_STRING));
}

#[test]
fn contains_can_fail_to_match() {
    assert!(!contains_ignore_case("xyz", TEST_STRING));
}

// --- EndsWith ---

#[test]
fn ends_with_ignores_case() {
    assert!(ends_with_ignore_case("string", TEST_STRING));
}

#[test]
fn ends_with_can_fail_to_match() {
    assert!(!ends_with_ignore_case("xyz", TEST_STRING));
}

// --- StartsWith ---

#[test]
fn starts_with_ignores_case() {
    assert!(starts_with_ignore_case("some", TEST_STRING));
}

#[test]
fn starts_with_can_fail_to_match() {
    assert!(!starts_with_ignore_case("xyz", TEST_STRING));
}
