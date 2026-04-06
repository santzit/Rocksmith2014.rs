//! Integration tests using real Rocksmith 2014 CDLC PSARC files.
//!
//! These tests verify that the library can correctly open, parse, and extract
//! content from actual Rocksmith 2014 archives.

use rocksmith2014_psarc::{NamedEntry, Psarc, PsarcError};
use std::io::Cursor;
use std::path::PathBuf;

/// Returns the path to the test CDLC directory.
fn cdlc_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("Rocksmith2014.PSARC.Tests")
        .join("cdlc")
}

/// Paths to all five real PSARC test files.
fn test_psarc_paths() -> Vec<PathBuf> {
    let dir = cdlc_dir();
    vec![
        dir.join("The-Cure-_In-Between-Days-_v2_p.psarc"),
        dir.join("The-Cure_A-Forest_v3_DD_p.psarc"),
        dir.join("Tom-Petty-and-the-Heartbreakers_Dont-Do-Me-Like-That_v2_p.psarc"),
        dir.join("Tom-Petty_Love-Is-A-Long-Road-Dell_v1_1_p.psarc"),
        dir.join("Tom-Petty_Runnin'-Down-a-Dream_v2_DD_p.psarc"),
    ]
}

// ---------------------------------------------------------------------------
// Reading tests
// ---------------------------------------------------------------------------

#[test]
fn test_open_all_cdlc_files() {
    for path in test_psarc_paths() {
        let psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        // Every CDLC should have at least one entry.
        assert!(
            !psarc.manifest().is_empty(),
            "{} has an empty manifest",
            path.display()
        );
    }
}

#[test]
fn test_manifest_contains_expected_paths() {
    for path in test_psarc_paths() {
        let psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        let manifest = psarc.manifest();

        // Rocksmith 2014 CDLC packages always contain artwork.
        let has_artwork = manifest.iter().any(|n| n.contains("album_") || n.contains("dlc_") || n.ends_with(".dds"));
        assert!(
            has_artwork,
            "{} manifest should contain artwork; manifest={:?}",
            path.display(),
            &manifest[..manifest.len().min(10)]
        );
    }
}

#[test]
fn test_manifest_entry_count_matches_toc() {
    for path in test_psarc_paths() {
        let psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        assert_eq!(
            psarc.manifest().len(),
            psarc.toc().len(),
            "{}: manifest length must equal TOC entry count",
            path.display()
        );
    }
}

#[test]
fn test_inflate_all_entries_produces_nonempty_data() {
    for path in test_psarc_paths() {
        let mut psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        let toc: Vec<_> = psarc.toc().to_vec();
        for entry in &toc {
            let data = psarc
                .inflate_entry(entry)
                .unwrap_or_else(|e| panic!("{}: inflate_entry failed: {}", path.display(), e));
            assert_eq!(
                data.len() as u64,
                entry.length,
                "{}: inflated length {} != declared length {}",
                path.display(),
                data.len(),
                entry.length
            );
        }
    }
}

#[test]
fn test_inflate_file_by_name() {
    for path in test_psarc_paths() {
        let mut psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        // Inflate the first entry by name to confirm name-based lookup works.
        let first_name = psarc.manifest()[0].clone();
        let data = psarc
            .inflate_file(&first_name)
            .unwrap_or_else(|e| panic!("{}: inflate_file({:?}) failed: {}", path.display(), first_name, e));
        assert!(!data.is_empty(), "{}: inflated first entry is empty", path.display());
    }
}

#[test]
fn test_inflate_file_not_found_returns_error() {
    let path = test_psarc_paths().into_iter().next().unwrap();
    let mut psarc = Psarc::open(&path).unwrap();

    let result = psarc.inflate_file("this_file_does_not_exist.xyz");
    assert!(
        matches!(result, Err(PsarcError::FileNotFound(_))),
        "expected FileNotFound, got {:?}",
        result
    );
}

#[test]
fn test_entry_offsets_are_unique() {
    for path in test_psarc_paths() {
        let psarc = Psarc::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {}", path.display(), e));

        let mut offsets: Vec<u64> = psarc.toc().iter().map(|e| e.offset).collect();
        offsets.sort_unstable();
        offsets.dedup();
        assert_eq!(
            offsets.len(),
            psarc.toc().len(),
            "{}: duplicate offsets found in TOC",
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Extract-all test
// ---------------------------------------------------------------------------

#[test]
fn test_extract_all() {
    let path = test_psarc_paths().into_iter().next().unwrap();
    let mut psarc = Psarc::open(&path).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    psarc
        .extract_all(tmp.path())
        .unwrap_or_else(|e| panic!("{}: extract_all failed: {}", path.display(), e));

    // At least one file should have been extracted.
    let mut found = false;
    for entry in walkdir(tmp.path()) {
        if entry.is_file() {
            found = true;
            break;
        }
    }
    assert!(found, "extract_all produced no files");
}

// ---------------------------------------------------------------------------
// Round-trip write → read tests
// ---------------------------------------------------------------------------

#[test]
fn test_round_trip_single_file_encrypted() {
    let original = b"Rocksmith 2014 test data".to_vec();
    let entries = vec![NamedEntry {
        name: "test/file.bin".to_string(),
        data: original.clone(),
    }];

    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let data = psarc.inflate_file("test/file.bin").unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_round_trip_multiple_files_unencrypted() {
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("songs/arr/lead.xml", b"<arrangement />".to_vec()),
        ("songs/audio/song.wem", vec![0xAB; 256]),
        ("appid", b"248750".to_vec()),
    ];

    let entries: Vec<NamedEntry> = files
        .iter()
        .map(|(name, data)| NamedEntry {
            name: name.to_string(),
            data: data.clone(),
        })
        .collect();

    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert_eq!(psarc.manifest().len(), 3);

    for (name, expected) in &files {
        let data = psarc.inflate_file(name).unwrap();
        assert_eq!(&data, expected, "mismatch for {}", name);
    }
}

#[test]
fn test_round_trip_large_file() {
    // A file that spans multiple 64 KB blocks.
    let original: Vec<u8> = (0u8..=255).cycle().take(300_000).collect();
    let entries = vec![NamedEntry {
        name: "data/large.bin".to_string(),
        data: original.clone(),
    }];

    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let data = psarc.inflate_file("data/large.bin").unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_round_trip_plain_sng_file() {
    // .sng files must not be re-compressed.
    let sng: Vec<u8> = (0u8..=127).cycle().take(1024).collect();
    let entries = vec![NamedEntry {
        name: "song.sng".to_string(),
        data: sng.clone(),
    }];

    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let data = psarc.inflate_file("song.sng").unwrap();
    assert_eq!(data, sng);
}

#[test]
fn test_round_trip_empty_entries_list() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, vec![]).unwrap();

    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
    assert!(psarc.toc().is_empty());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
