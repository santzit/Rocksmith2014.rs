//! Tests mirroring Rocksmith2014.PSARC.Tests/EditTests.fs

use rocksmith2014_psarc::{NamedEntry, Psarc};
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
