//! Tests that mirror the official Rocksmith2014.NET PSARC test suite.
//!
//! The .NET project has two test modules:
//! - `ReadTests.fs`  — opening, manifest access, extraction
//! - `EditTests.fs`  — editing (add/remove/reorder/rename entries)
//!
//! These tests use the same three PSARC test files that the .NET tests use,
//! downloaded from the Rocksmith2014.NET repository.

use rocksmith2014_psarc::{NamedEntry, Psarc};
use std::io::Cursor;
use std::path::PathBuf;

/// Returns the path to the .NET-sourced test PSARC files.
fn psarc_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("psarc")
}

// =============================================================================
// Read Tests  (mirrors ReadTests.fs)
// =============================================================================

/// "Can read PSARC with encrypted TOC"
///
/// .NET: `Expect.equal psarc.Manifest.[0] "gfxassets/album_art/album_testtest_64.dds"`
#[test]
fn test_read_encrypted_toc() {
    let path = psarc_dir().join("test_p.psarc");
    let psarc = Psarc::open(&path).expect("failed to open test_p.psarc");

    assert_eq!(
        psarc.manifest()[0],
        "gfxassets/album_art/album_testtest_64.dds",
        "First manifest entry should match"
    );
}

/// "Can extract all files from PSARC"
///
/// .NET: `Expect.equal fileCount psarc.TOC.Count "All files were extracted"`
#[test]
fn test_extract_all_files() {
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
///
/// .NET: single-entry PSARC where the first block is zlib-compressed but
/// subsequent blocks are stored raw.  Extracting should succeed.
///
/// .NET: `Expect.equal fileCount psarc.TOC.Count "One file was extracted"`
#[test]
fn test_extract_partially_compressed_file() {
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

// =============================================================================
// Edit Tests  (mirrors EditTests.fs)
// =============================================================================

/// Helper: load `test_edit_p.psarc` into memory so we can test round-trips.
fn load_test_edit() -> Psarc<Cursor<Vec<u8>>> {
    let path = psarc_dir().join("test_edit_p.psarc");
    let bytes = std::fs::read(&path).expect("failed to read test_edit_p.psarc");
    Psarc::read(Cursor::new(bytes)).expect("failed to parse test_edit_p.psarc")
}

/// Helper: apply a transformation function, write the result to a fresh buffer,
/// then re-open the new archive.  Mirrors the .NET pattern of `Edit + re-read`.
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
///
/// .NET: `Expect.sequenceContainsOrder psarc.Manifest oldManifest "Manifest is unchanged"`
#[test]
fn test_null_edit_manifest_unchanged() {
    let mut psarc = load_test_edit();
    let old_manifest: Vec<String> = psarc.manifest().to_vec();

    let edited = apply_edit(&mut psarc, true, |entries| entries); // identity

    assert_eq!(
        edited.manifest(),
        old_manifest.as_slice(),
        "Manifest should be unchanged after a null edit"
    );
}

/// "Can be read after editing"
///
/// .NET:
/// ```fsharp
/// Expect.sequenceEqual psarc2.Manifest oldManifest "Manifest is unchanged"
/// for i = 0 to psarc2.TOC.Count - 1 do
///     Expect.equal psarc2.TOC.[i].Length oldToc.[i].Length "File length is same"
/// ```
#[test]
fn test_can_be_read_after_editing() {
    let mut psarc = load_test_edit();
    let old_manifest: Vec<String> = psarc.manifest().to_vec();
    let old_lengths: Vec<u64> = psarc.toc().iter().map(|e| e.length).collect();

    let edited = apply_edit(&mut psarc, true, |entries| entries); // identity

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
///
/// .NET: remove all `.wem` entries (there are 2) and check manifest is 2 shorter.
///
/// .NET:
/// ```fsharp
/// do! psarc.Edit(options, (List.filter (fun x -> not <| x.Name.EndsWith "wem")))
/// Expect.equal psarc2.Manifest.Length (oldManifest.Length - 2) "Manifest size is correct"
/// Expect.isTrue (memory.Length < oldSize) "Size is smaller"
/// ```
#[test]
fn test_can_remove_files() {
    let mut psarc = load_test_edit();
    let old_count = psarc.manifest().len();

    let edited = apply_edit(&mut psarc, true, |entries| {
        entries.into_iter().filter(|e| !e.name.ends_with(".wem")).collect()
    });

    assert_eq!(
        edited.manifest().len(),
        old_count - 2,
        "Manifest should have 2 fewer entries after removing .wem files"
    );
}

/// "Can add a file"
///
/// .NET:
/// ```fsharp
/// do! psarc.Edit(options, (fun files -> fileToAdd :: files))
/// Expect.equal psarc.Manifest.Length (oldManifest.Length + 1) "Manifest size is correct"
/// Expect.equal psarc.Manifest.[0] fileToAdd.Name "Name in manifest is correct"
/// ```
#[test]
fn test_can_add_file() {
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
        edited.manifest()[0], new_name,
        "New file name should be first in manifest"
    );
}

/// "Can reorder files"
///
/// .NET:
/// ```fsharp
/// let first = List.head files
/// (List.tail files) @ [ first ]
/// Expect.equal psarc.Manifest.[psarc.Manifest.Length - 1] oldManifest.[0] "First file is now last"
/// ```
#[test]
fn test_can_reorder_files() {
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
///
/// .NET:
/// ```fsharp
/// do! psarc.Edit(options, List.mapi (fun i item ->
///     if i = 0 then { item with Name = "new name" } else item))
/// Expect.equal psarc.Manifest.[0] "new name" "File name is changed"
/// ```
#[test]
fn test_can_rename_files() {
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
        edited.manifest()[0], "new name",
        "First file name should be changed to 'new name'"
    );
}

// =============================================================================
// Helpers
// =============================================================================

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
