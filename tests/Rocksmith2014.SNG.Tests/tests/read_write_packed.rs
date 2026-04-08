//! Mirrors ReadWritePacked.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::{Platform, Sng};
use std::path::PathBuf;

fn sng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const TEST_FILE_LEVELS: usize = 12;

#[test]
fn test_can_read_packed_pc_sng() {
    let data = std::fs::read(sng_dir().join("packed_pc.sng")).expect("read packed_pc.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("decrypt PC SNG");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "packed_pc.sng should have {} levels", TEST_FILE_LEVELS);
}

#[test]
fn test_can_read_packed_mac_sng() {
    let data = std::fs::read(sng_dir().join("packed_mac.sng")).expect("read packed_mac.sng");
    let sng = Sng::from_encrypted(&data, Platform::Mac).expect("decrypt Mac SNG");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "packed_mac.sng should have {} levels", TEST_FILE_LEVELS);
}

#[test]
fn test_can_write_packed_pc_sng() {
    let data = std::fs::read(sng_dir().join("packed_pc.sng")).expect("read packed_pc.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("decrypt PC SNG");
    let rewritten = sng.to_encrypted(Platform::Pc).expect("re-encrypt PC SNG");
    let sng2 = Sng::from_encrypted(&rewritten, Platform::Pc).expect("re-read PC SNG");
    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "written packed_pc.sng should still have {} levels", TEST_FILE_LEVELS);
}

#[test]
fn test_can_write_packed_mac_sng() {
    let data = std::fs::read(sng_dir().join("packed_mac.sng")).expect("read packed_mac.sng");
    let sng = Sng::from_encrypted(&data, Platform::Mac).expect("decrypt Mac SNG");
    let rewritten = sng.to_encrypted(Platform::Mac).expect("re-encrypt Mac SNG");
    let sng2 = Sng::from_encrypted(&rewritten, Platform::Mac).expect("re-read Mac SNG");
    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "written packed_mac.sng should still have {} levels", TEST_FILE_LEVELS);
}
