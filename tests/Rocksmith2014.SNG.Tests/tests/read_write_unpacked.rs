//! Mirrors ReadWriteUnpacked.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::Sng;
use std::path::PathBuf;

fn sng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const TEST_FILE_LEVELS: usize = 12;

#[test]
fn test_can_read_unpacked_sng() {
    let data = std::fs::read(sng_dir().join("unpacked.sng")).expect("read unpacked.sng");
    let sng = Sng::read(&data).expect("parse unpacked SNG");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "unpacked.sng should have {} levels", TEST_FILE_LEVELS);
}

#[test]
fn test_can_write_unpacked_sng() {
    let original = std::fs::read(sng_dir().join("unpacked.sng")).expect("read unpacked.sng");
    let sng = Sng::read(&original).expect("parse unpacked SNG");
    let rewritten = sng.write().expect("write unpacked SNG");
    let sng2 = Sng::read(&rewritten).expect("re-read written SNG");
    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "written unpacked.sng should have {} levels", TEST_FILE_LEVELS);
    assert_eq!(rewritten.len(), original.len(),
        "written file should be the same size as the original");
}
