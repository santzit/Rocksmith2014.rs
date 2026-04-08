//! Tests mirroring Rocksmith2014.PSARC.Tests/EditTests.fs

use rocksmith2014_psarc::{NamedEntry, Psarc, PsarcError};
use std::io::Cursor;
use std::path::PathBuf;

fn psarc_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_test_edit() -> Psarc<Cursor<Vec<u8>>> {
    let path = psarc_dir().join("test_edit_p.psarc");
    let bytes = std::fs::read(&path).expect("failed to read test_edit_p.psarc");
    Psarc::read(Cursor::new(bytes)).expect("failed to parse test_edit_p.psarc")
}

fn apply_edit<F>(
    psarc: &mut Psarc<Cursor<Vec<u8>>>,
    encrypt: bool,
    edit_fn: F,
) -> Psarc<Cursor<Vec<u8>>>
where
    F: FnOnce(Vec<NamedEntry>) -> Vec<NamedEntry>,
{
    let entries = psarc
        .read_all_named_entries()
        .expect("read_all_named_entries failed");
    let new_entries = edit_fn(entries);

    let mut buf = Vec::new();
    Psarc::create(&mut buf, encrypt, new_entries).expect("Psarc::create failed");
    Psarc::read(Cursor::new(buf)).expect("re-read after edit failed")
}

/// "Manifest is same after null edit"
#[test]
fn manifest_is_same_after_null_edit() {
    let mut psarc = load_test_edit();
    let old_manifest: Vec<String> = psarc.manifest().to_vec();

    let edited = apply_edit(&mut psarc, true, |entries| entries);

    assert_eq!(
        edited.manifest(),
        old_manifest.as_slice(),
        "Manifest should be unchanged after a null edit"
    );
}

/// "Can be read after editing"
#[test]
fn can_be_read_after_editing() {
    let mut psarc = load_test_edit();
    let old_manifest: Vec<String> = psarc.manifest().to_vec();
    let old_lengths: Vec<u64> = psarc.toc().iter().map(|e| e.length).collect();

    let edited = apply_edit(&mut psarc, true, |entries| entries);

    assert_eq!(
        edited.manifest(),
        old_manifest.as_slice(),
        "Manifest should be unchanged after re-read"
    );
    let new_lengths: Vec<u64> = edited.toc().iter().map(|e| e.length).collect();
    assert_eq!(
        new_lengths, old_lengths,
        "All file lengths should be unchanged after a null edit"
    );
}

/// "Can remove files"
#[test]
fn can_remove_files() {
    let mut psarc = load_test_edit();
    let old_count = psarc.manifest().len();

    let edited = apply_edit(&mut psarc, true, |entries| {
        entries
            .into_iter()
            .filter(|e| !e.name.ends_with(".wem"))
            .collect()
    });

    assert_eq!(
        edited.manifest().len(),
        old_count - 2,
        "Manifest should have 2 fewer entries after removing .wem files"
    );
}

/// "Can add a file"
#[test]
fn can_add_a_file() {
    let mut psarc = load_test_edit();
    let old_count = psarc.manifest().len();
    let new_name = "test/new_file.bin";

    let edited = apply_edit(&mut psarc, true, |mut entries| {
        entries.insert(
            0,
            NamedEntry {
                name: new_name.to_string(),
                data: b"hello from new file".to_vec(),
            },
        );
        entries
    });

    assert_eq!(
        edited.manifest().len(),
        old_count + 1,
        "Manifest should have 1 more entry after adding a file"
    );
    assert_eq!(
        edited.manifest()[0],
        new_name,
        "New file name should be first in manifest"
    );
}

/// "Can reorder files"
#[test]
fn can_reorder_files() {
    let mut psarc = load_test_edit();
    let original_first = psarc.manifest()[0].clone();
    let old_count = psarc.manifest().len();

    let edited = apply_edit(&mut psarc, true, |mut entries| {
        let first = entries.remove(0);
        entries.push(first);
        entries
    });

    assert_eq!(
        edited.manifest().len(),
        old_count,
        "Manifest size should be unchanged after reordering"
    );
    assert_eq!(
        edited.manifest()[old_count - 1],
        original_first,
        "The original first file should now be last"
    );
}

/// "Can rename files"
#[test]
fn can_rename_files() {
    let mut psarc = load_test_edit();
    let old_count = psarc.manifest().len();

    let edited = apply_edit(&mut psarc, true, |mut entries| {
        if let Some(first) = entries.first_mut() {
            first.name = "new name".to_string();
        }
        entries
    });

    assert_eq!(
        edited.manifest().len(),
        old_count,
        "Manifest size should be unchanged after renaming"
    );
    assert_eq!(
        edited.manifest()[0],
        "new name",
        "First file name should be changed to 'new name'"
    );
}

// ---------------------------------------------------------------------------
// Round-trip write → read tests
// ---------------------------------------------------------------------------

#[test]
fn round_trip_single_file_encrypted() {
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
fn round_trip_multiple_files_unencrypted() {
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
fn round_trip_large_file() {
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
fn round_trip_sng_file_not_recompressed() {
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
fn round_trip_empty_entries_list() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, vec![]).unwrap();

    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
    assert!(psarc.toc().is_empty());
}

#[test]
fn inflate_file_not_found_returns_error() {
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
fn non_sng_entry_is_zlib_compressed() {
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
fn sng_entry_stored_uncompressed() {
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
fn inflate_entry_by_toc_index() {
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
fn round_trip_multiple_named_files() {
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
fn round_trip_single_file_unencrypted() {
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
fn round_trip_single_file_encrypted_separate() {
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
fn empty_archive_unencrypted() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, false, vec![]).unwrap();
    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
}

#[test]
fn empty_archive_encrypted() {
    let mut buf = Vec::new();
    Psarc::create(&mut buf, true, vec![]).unwrap();
    let psarc = Psarc::read(Cursor::new(&buf)).unwrap();
    assert!(psarc.manifest().is_empty());
}
