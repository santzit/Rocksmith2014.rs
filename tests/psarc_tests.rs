//! PSARC tests mirroring Rocksmith2014.PSARC.Tests from Rocksmith2014.NET v3.5.0.
//!
//! Source .NET test files:
//!  - ReadTests.fs  – 3 tests
//!  - EditTests.fs  – 6 tests
//!
//! Test data: tests/cdlc/test_p.psarc, test_edit_p.psarc, partially_compressed_test_p.psarc

use rocksmith2014::psarc::{Psarc, PsarcBuilder};

fn cdlc(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cdlc")
        .join(name)
}

// ===========================================================================
// ReadTests.fs  (3 tests)
// ===========================================================================

/// Mirrors: ReadTests."Can read PSARC with encrypted TOC"
#[test]
fn psarc_read_can_read_with_encrypted_toc() {
    let arc = Psarc::open(cdlc("test_p.psarc")).expect("open test_p.psarc");
    assert_eq!(
        arc.entry_names()[0],
        "gfxassets/album_art/album_testtest_64.dds",
        "First file name is correct"
    );
}

/// Mirrors: ReadTests."Can extract all files from PSARC"
#[test]
fn psarc_read_can_extract_all_files() {
    let arc = Psarc::open(cdlc("test_p.psarc")).expect("open test_p.psarc");
    let count = arc.entry_count();
    for i in 0..count {
        arc.extract(i).unwrap_or_else(|_| panic!("extract entry {i}"));
    }
    assert!(count > 0, "At least one entry was extracted");
}

/// Mirrors: ReadTests."Can extract partially compressed file"
#[test]
fn psarc_read_can_extract_partially_compressed() {
    let arc = Psarc::open(cdlc("partially_compressed_test_p.psarc"))
        .expect("open partially_compressed_test_p.psarc");
    let count = arc.entry_count();
    assert!(count > 0, "Archive has entries");
    for i in 0..count {
        arc.extract(i).unwrap_or_else(|_| panic!("extract entry {i}"));
    }
}

// ===========================================================================
// EditTests.fs  (6 tests)
// ===========================================================================

/// Mirrors: EditTests."Manifest is same after null edit"
///
/// Read a PSARC, rebuild it with the same entries (identity/null edit), then
/// re-read and verify the manifest is unchanged.
#[test]
fn psarc_edit_manifest_same_after_null_edit() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_names: Vec<String> = arc.entry_names().to_vec();

    // Null edit: rebuild with the same entries in the same order
    let rebuilt = rebuild_psarc(&arc);
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read after null edit");

    assert_eq!(arc2.entry_names(), old_names.as_slice(),
        "Manifest unchanged after null edit");
}

/// Mirrors: EditTests."Can be read after editing"
///
/// After rebuilding, re-read and verify manifest and file lengths are equal.
#[test]
fn psarc_edit_can_be_read_after_editing() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_names: Vec<String> = arc.entry_names().to_vec();
    let old_lengths: Vec<usize> = (0..arc.entry_count())
        .map(|i| arc.extract(i).unwrap().len())
        .collect();

    // Null edit: rebuild, then re-read
    let rebuilt = rebuild_psarc(&arc);
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read rebuilt PSARC");

    assert_eq!(arc2.entry_names(), old_names.as_slice(),
        "Manifest is unchanged after editing");
    for (i, (&expected_len, name)) in old_lengths.iter().zip(old_names.iter()).enumerate() {
        let actual_len = arc2.extract(i).unwrap().len();
        assert_eq!(actual_len, expected_len,
            "File length for '{name}' is same after editing");
    }
}

/// Mirrors: EditTests."Can remove files"
///
/// Remove all files ending in "wem" and verify the manifest shrinks by the
/// correct amount.
#[test]
fn psarc_edit_can_remove_files() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_count = arc.entry_count();

    let wem_count = arc.entry_names().iter().filter(|n| n.ends_with("wem")).count();
    assert!(wem_count > 0, "test_edit_p.psarc should contain .wem entries");

    // Rebuild without entries ending in "wem"
    let mut builder = PsarcBuilder::new();
    for i in 0..arc.entry_count() {
        let name = arc.entry_name(i).unwrap().to_owned();
        if !name.ends_with("wem") {
            builder.add_entry(name, arc.extract(i).unwrap());
        }
    }
    let rebuilt = builder.build();
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read after remove");

    assert_eq!(arc2.entry_count(), old_count - wem_count,
        "Manifest size is correct after removal");
}

/// Mirrors: EditTests."Can add a file"
///
/// Add a new dummy entry and verify the manifest grows by one.
#[test]
fn psarc_edit_can_add_a_file() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_count = arc.entry_count();

    let new_name = "test/test_added.bin";
    let new_data = b"hello rocksmith".to_vec();

    // Rebuild with one extra entry prepended (index 0, matches .NET "first" behaviour)
    let mut builder = PsarcBuilder::new();
    builder.add_entry(new_name, new_data);
    for i in 0..arc.entry_count() {
        builder.add_entry(arc.entry_name(i).unwrap(), arc.extract(i).unwrap());
    }
    let rebuilt = builder.build();
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read after add");

    assert_eq!(arc2.entry_count(), old_count + 1,
        "Manifest size is correct after adding a file");
    assert_eq!(arc2.entry_name(0).unwrap(), new_name,
        "Name in manifest is correct");
}

/// Mirrors: EditTests."Can reorder files"
///
/// Move the first entry to the end and verify the manifest wraps correctly.
#[test]
fn psarc_edit_can_reorder_files() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_names: Vec<String> = arc.entry_names().to_vec();
    let old_count = arc.entry_count();

    // Rotate: move first to last
    let mut builder = PsarcBuilder::new();
    for i in 1..arc.entry_count() {
        builder.add_entry(arc.entry_name(i).unwrap(), arc.extract(i).unwrap());
    }
    builder.add_entry(arc.entry_name(0).unwrap(), arc.extract(0).unwrap());
    let rebuilt = builder.build();
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read after reorder");

    assert_eq!(arc2.entry_count(), old_count, "Manifest size is same");
    assert_eq!(
        arc2.entry_name(old_count - 1).unwrap(),
        old_names[0].as_str(),
        "First file is now last"
    );
}

/// Mirrors: EditTests."Can rename files"
///
/// Rename the first entry and verify the change is reflected.
#[test]
fn psarc_edit_can_rename_files() {
    let arc = Psarc::open(cdlc("test_edit_p.psarc")).expect("open test_edit_p.psarc");
    let old_count = arc.entry_count();
    let new_name = "new name";

    // Rebuild with first entry renamed
    let mut builder = PsarcBuilder::new();
    builder.add_entry(new_name, arc.extract(0).unwrap());
    for i in 1..arc.entry_count() {
        builder.add_entry(arc.entry_name(i).unwrap(), arc.extract(i).unwrap());
    }
    let rebuilt = builder.build();
    let arc2 = Psarc::from_bytes(rebuilt).expect("re-read after rename");

    assert_eq!(arc2.entry_count(), old_count, "Manifest size is same after rename");
    assert_eq!(arc2.entry_name(0).unwrap(), new_name, "File name is changed");
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Rebuild a PSARC from its own entries (null / identity edit).
fn rebuild_psarc(arc: &Psarc) -> Vec<u8> {
    let mut builder = PsarcBuilder::new();
    for i in 0..arc.entry_count() {
        builder.add_entry(arc.entry_name(i).unwrap(), arc.extract(i).unwrap());
    }
    builder.build()
}
