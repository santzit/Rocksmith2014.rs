//! Additional integration tests for the rocksmith2014-psarc crate.
//!
//! These tests focus on pure in-memory round-trip write → read scenarios that
//! complement the .NET-parity tests in `dotnet_parity_tests.rs`.  No CDLC
//! or third-party PSARC files are used here.

use rocksmith2014_psarc::{NamedEntry, Psarc, PsarcError};
use std::io::Cursor;

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

#[test]
fn test_inflate_file_not_found_returns_error() {
    let entries = vec![NamedEntry {
        name: "existing.bin".to_string(),
        data: b"data".to_vec(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let result = psarc.inflate_file("this_file_does_not_exist.xyz");
    assert!(
        matches!(result, Err(PsarcError::FileNotFound(_))),
        "expected FileNotFound, got {:?}",
        result
    );
}

#[test]
fn test_has_zlib_header() {
    // Entries written into a new PSARC (non-.sng) should be zlib-compressed,
    // and inflating them should produce the original data unchanged.
    let data: Vec<u8> = b"Hello, Rocksmith!".to_vec();
    let entries = vec![NamedEntry {
        name: "readme.txt".to_string(),
        data: data.clone(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let inflated = psarc.inflate_file("readme.txt").unwrap();
    assert_eq!(inflated, data);
}

#[test]
fn test_plain_sng_file_not_compressed() {
    // Entries whose name ends in ".sng" must be stored uncompressed inside
    // the PSARC (block-size entry == 0 means raw / uncompressed).
    let sng: Vec<u8> = vec![0x4A; 128];
    let entries = vec![NamedEntry {
        name: "songs/bin/generic/test.sng".to_string(),
        data: sng.clone(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let inflated = psarc.inflate_file("songs/bin/generic/test.sng").unwrap();
    assert_eq!(inflated, sng);
}

#[test]
fn test_inflate_entry() {
    let original = b"entry data payload".to_vec();
    let entries = vec![NamedEntry {
        name: "payload.bin".to_string(),
        data: original.clone(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let entry = psarc.toc()[0].clone();
    let data = psarc.inflate_entry(&entry).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_multiple_files() {
    let names = ["a.txt", "b.txt", "c.txt", "d.txt"];
    let entries: Vec<NamedEntry> = names
        .iter()
        .map(|n| NamedEntry {
            name: n.to_string(),
            data: n.as_bytes().to_vec(),
        })
        .collect();

    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert_eq!(psarc.manifest().len(), 4);

    for name in &names {
        let data = psarc.inflate_file(name).unwrap();
        assert_eq!(data, name.as_bytes());
    }
}

#[test]
fn test_single_text_file_unencrypted() {
    let text = b"Hello, world!".to_vec();
    let entries = vec![NamedEntry {
        name: "hello.txt".to_string(),
        data: text.clone(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let data = psarc.inflate_file("hello.txt").unwrap();
    assert_eq!(data, text);
}

#[test]
fn test_single_text_file_encrypted() {
    let text = b"Hello, encrypted world!".to_vec();
    let entries = vec![NamedEntry {
        name: "hello.txt".to_string(),
        data: text.clone(),
    }];
    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, entries).unwrap();

    let mut psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    let data = psarc.inflate_file("hello.txt").unwrap();
    assert_eq!(data, text);
}

#[test]
fn test_empty_archive_unencrypted() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, vec![]).unwrap();
    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
}

#[test]
fn test_empty_archive_encrypted() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, vec![]).unwrap();
    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
}

