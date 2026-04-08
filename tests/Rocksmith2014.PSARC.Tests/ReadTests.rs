//! Tests mirroring Rocksmith2014.PSARC.Tests/ReadTests.fs

use rocksmith2014_psarc::Psarc;
use std::path::PathBuf;

fn psarc_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn count_files(dir: &std::path::Path) -> usize {
    walkdir(dir).into_iter().filter(|p| p.is_file()).count()
}

fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walkdir(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

/// "Can read PSARC with encrypted TOC"
#[test]
fn can_read_psarc_with_encrypted_toc() {
    let path = psarc_dir().join("test_p.psarc");
    let psarc = Psarc::open(&path).expect("failed to open test_p.psarc");

    assert_eq!(
        psarc.manifest()[0],
        "gfxassets/album_art/album_testtest_64.dds",
        "First manifest entry should match"
    );
}

/// "Can extract all files from PSARC"
#[test]
fn can_extract_all_files() {
    let path = psarc_dir().join("test_p.psarc");
    let mut psarc = Psarc::open(&path).expect("failed to open test_p.psarc");

    let tmp = tempfile::tempdir().unwrap();
    psarc.extract_all(tmp.path()).expect("extract_all failed");

    let extracted = count_files(tmp.path());
    assert_eq!(
        extracted,
        psarc.toc().len(),
        "Extracted file count should equal TOC entry count"
    );
}

/// "Can extract partially compressed file"
#[test]
fn can_extract_partially_compressed_file() {
    let path = psarc_dir().join("partially_compressed_test_p.psarc");
    let mut psarc = Psarc::open(&path).expect("failed to open partially_compressed_test_p.psarc");

    let tmp = tempfile::tempdir().unwrap();
    psarc.extract_all(tmp.path()).expect("extract_all failed");

    let extracted = count_files(tmp.path());
    assert_eq!(
        extracted,
        psarc.toc().len(),
        "Extracted file count should equal TOC entry count"
    );
}
