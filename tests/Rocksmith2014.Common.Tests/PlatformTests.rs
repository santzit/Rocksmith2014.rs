//! Platform path tests.
//!
//! Mirrors `PlatformTests.fs` in Rocksmith2014.Common.Tests (.NET).

use rocksmith2014_common::Platform;

/// test "Path parts are correct for PC"
///
/// Expect.sequenceContainsOrder paths [ "windows"; "generic"; "_p" ] "Path parts are correct"
#[test]
fn path_parts_are_correct_for_pc() {
    let audio = Platform::Pc.audio_path();
    let sng = Platform::Pc.sng_path();
    let suffix = Platform::Pc.package_suffix();

    assert_eq!(audio, "windows");
    assert_eq!(sng, "generic");
    assert_eq!(suffix, "_p");
}

/// test "Path parts are correct for Mac"
///
/// Expect.sequenceContainsOrder paths [ "mac"; "macos"; "_m" ] "Path parts are correct"
#[test]
fn path_parts_are_correct_for_mac() {
    let audio = Platform::Mac.audio_path();
    let sng = Platform::Mac.sng_path();
    let suffix = Platform::Mac.package_suffix();

    assert_eq!(audio, "mac");
    assert_eq!(sng, "macos");
    assert_eq!(suffix, "_m");
}
