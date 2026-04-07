//! Integration tests using real SNG test data copied from Rocksmith2014.NET.
//!
//! Mirrors the .NET test suite:
//! - `ReadWritePacked.fs`   — packed (AES-CTR encrypted) PC and Mac files
//! - `ReadWriteUnpacked.fs` — raw binary (unencrypted) SNG file

use rocksmith2014_sng::{Platform, Sng};
use std::path::PathBuf;

/// Returns the path to the SNG test data directory.
fn sng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The number of difficulty levels present in every test SNG file.
const TEST_FILE_LEVELS: usize = 12;

// ---------------------------------------------------------------------------
// Packed (encrypted) files — mirrors ReadWritePacked.fs
// ---------------------------------------------------------------------------

/// "Can read packed PC SNG file"
#[test]
fn test_can_read_packed_pc_sng() {
    let data = std::fs::read(sng_dir().join("packed_pc.sng")).expect("read packed_pc.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("decrypt PC SNG");
    assert_eq!(
        sng.levels.len(),
        TEST_FILE_LEVELS,
        "packed_pc.sng should have {} levels",
        TEST_FILE_LEVELS
    );
}

/// "Can read packed Mac SNG file"
#[test]
fn test_can_read_packed_mac_sng() {
    let data = std::fs::read(sng_dir().join("packed_mac.sng")).expect("read packed_mac.sng");
    let sng = Sng::from_encrypted(&data, Platform::Mac).expect("decrypt Mac SNG");
    assert_eq!(
        sng.levels.len(),
        TEST_FILE_LEVELS,
        "packed_mac.sng should have {} levels",
        TEST_FILE_LEVELS
    );
}

/// "Can write packed PC SNG file"
///
/// Reads packed_pc.sng, re-encrypts it, then reads the result and confirms
/// the level count is unchanged.
#[test]
fn test_can_write_packed_pc_sng() {
    let data = std::fs::read(sng_dir().join("packed_pc.sng")).expect("read packed_pc.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("decrypt PC SNG");

    let rewritten = sng.to_encrypted(Platform::Pc).expect("re-encrypt PC SNG");
    let sng2 = Sng::from_encrypted(&rewritten, Platform::Pc).expect("re-read PC SNG");

    assert_eq!(
        sng2.levels.len(),
        TEST_FILE_LEVELS,
        "written packed_pc.sng should still have {} levels",
        TEST_FILE_LEVELS
    );
}

/// "Can write packed Mac SNG file"
///
/// Reads packed_mac.sng, re-encrypts it, then reads the result and confirms
/// the level count is unchanged.
#[test]
fn test_can_write_packed_mac_sng() {
    let data = std::fs::read(sng_dir().join("packed_mac.sng")).expect("read packed_mac.sng");
    let sng = Sng::from_encrypted(&data, Platform::Mac).expect("decrypt Mac SNG");

    let rewritten = sng.to_encrypted(Platform::Mac).expect("re-encrypt Mac SNG");
    let sng2 = Sng::from_encrypted(&rewritten, Platform::Mac).expect("re-read Mac SNG");

    assert_eq!(
        sng2.levels.len(),
        TEST_FILE_LEVELS,
        "written packed_mac.sng should still have {} levels",
        TEST_FILE_LEVELS
    );
}

// ---------------------------------------------------------------------------
// Unpacked (raw binary) files — mirrors ReadWriteUnpacked.fs
// ---------------------------------------------------------------------------

/// "Can read unpacked SNG file"
#[test]
fn test_can_read_unpacked_sng() {
    let data = std::fs::read(sng_dir().join("unpacked.sng")).expect("read unpacked.sng");
    let sng = Sng::read(&data).expect("parse unpacked SNG");
    assert_eq!(
        sng.levels.len(),
        TEST_FILE_LEVELS,
        "unpacked.sng should have {} levels",
        TEST_FILE_LEVELS
    );
}

/// "Can write unpacked SNG file"
///
/// Reads unpacked.sng, serialises it back, re-reads and confirms the level
/// count and byte-length are identical to the original.
#[test]
fn test_can_write_unpacked_sng() {
    let original = std::fs::read(sng_dir().join("unpacked.sng")).expect("read unpacked.sng");
    let sng = Sng::read(&original).expect("parse unpacked SNG");

    let rewritten = sng.write().expect("write unpacked SNG");
    let sng2 = Sng::read(&rewritten).expect("re-read written SNG");

    assert_eq!(
        sng2.levels.len(),
        TEST_FILE_LEVELS,
        "written unpacked.sng should have {} levels",
        TEST_FILE_LEVELS
    );
    assert_eq!(
        rewritten.len(),
        original.len(),
        "written file should be the same size as the original"
    );
}

// ---------------------------------------------------------------------------
// Negative array count guard — mirrors Bug 2 fix
// ---------------------------------------------------------------------------

/// Crafted input where the first i32 (beat array count) is -1.
/// Before the fix this caused a capacity overflow / OOM panic via
/// `Vec::with_capacity((-1i32) as usize)`.
/// After the fix it must return `Err(Error::InvalidArrayCount(-1))`.
#[test]
fn test_negative_array_count_returns_error() {
    use rocksmith2014_sng::Error;

    // First 4 bytes = array count as little-endian i32 = -1
    let crafted: Vec<u8> = (-1i32).to_le_bytes().to_vec();
    let result = Sng::read(&crafted);
    match result {
        Err(Error::InvalidArrayCount(n)) => assert_eq!(n, -1),
        other => panic!("expected InvalidArrayCount(-1), got {:?}", other),
    }
}
