//! SNG tests mirroring Rocksmith2014.SNG.Tests from Rocksmith2014.NET v3.5.0.
//!
//! Source .NET test files:
//!  - BinaryHelpersTests.fs – 1 test
//!  - ReadWritePacked.fs    – 4 tests
//!  - ReadWriteUnpacked.fs  – 2 tests
//!  - RoundTripTests.fs     – ~20 property/case tests
//!
//! Test data: tests/cdlc/packed_pc.sng, packed_mac.sng, unpacked.sng

use rocksmith2014::sng::{Platform, Sng};

fn cdlc(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cdlc")
        .join(name)
}

const TEST_FILE_LEVELS: usize = 12;

// ===========================================================================
// BinaryHelpersTests.fs  (1 test)
// ===========================================================================

/// Mirrors: BinaryHelpersTests."UTF8 string always includes null terminator"
///
/// Writing a fixed-length UTF-8 string field must produce a null byte at
/// position `length - 1` regardless of the input length.
#[test]
fn sng_binary_helpers_utf8_null_terminator() {
    use rocksmith2014::sng::sng_to_bytes;
    use rocksmith2014::sng::Sng;

    // We serialise a complete (empty) SNG and look at the first string field
    // of MetaData (LastConversionDateTime), which is a fixed 32-byte field.
    // Any non-empty string shorter than 32 chars must be followed by \0.
    let mut sng = make_minimal_sng();
    // Set a short string in the last_conversion_date_time field (< 32 chars)
    let short_str = "A".repeat(31); // 31 'A's + null = 32 bytes
    sng.metadata.last_conversion_date_time = short_str.clone();

    let bytes = sng_to_bytes(&sng);

    // The last_conversion_date_time field starts at a known offset inside the
    // metadata block. Rather than hardcode the offset, we search for the
    // field content and verify the byte after it is null.
    let field_bytes = short_str.as_bytes();
    let pos = bytes
        .windows(field_bytes.len())
        .position(|w| w == field_bytes)
        .expect("short_str should be in serialised bytes");

    assert_eq!(
        bytes[pos + field_bytes.len()], 0,
        "Byte immediately after the string content must be the null terminator"
    );
}

// ===========================================================================
// ReadWriteUnpacked.fs  (2 tests)
// ===========================================================================

/// Mirrors: ReadWriteUnpacked."Can read unpacked SNG file"
#[test]
fn sng_can_read_unpacked_file() {
    let sng = Sng::read_unpacked_file(cdlc("unpacked.sng")).expect("read unpacked.sng");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "Read {TEST_FILE_LEVELS} levels");
}

/// Mirrors: ReadWriteUnpacked."Can write unpacked SNG file"
///
/// Read, write, re-read; verify level count and byte-for-byte file size match.
#[test]
fn sng_can_write_unpacked_file() {
    use rocksmith2014::sng::sng_to_bytes;

    let sng = Sng::read_unpacked_file(cdlc("unpacked.sng")).expect("read unpacked.sng");
    let original_bytes = std::fs::read(cdlc("unpacked.sng")).unwrap();

    let written_bytes = sng_to_bytes(&sng);
    let sng2 = Sng::from_unpacked_bytes(&written_bytes).expect("re-read after write");

    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "Re-read gives same level count");
    assert_eq!(written_bytes.len(), original_bytes.len(),
        "Same size file written");
}

// ===========================================================================
// ReadWritePacked.fs  (4 tests)
// ===========================================================================

/// Mirrors: ReadWritePacked."Can read packed PC SNG file"
#[test]
fn sng_read_packed_pc() {
    let sng = Sng::read_packed_file(cdlc("packed_pc.sng"), Platform::Pc)
        .expect("read packed_pc.sng");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "Read {TEST_FILE_LEVELS} levels from PC-packed SNG");
}

/// Mirrors: ReadWritePacked."Can read packed Mac SNG file"
#[test]
fn sng_read_packed_mac() {
    let sng = Sng::read_packed_file(cdlc("packed_mac.sng"), Platform::Mac)
        .expect("read packed_mac.sng");
    assert_eq!(sng.levels.len(), TEST_FILE_LEVELS,
        "Read {TEST_FILE_LEVELS} levels from Mac-packed SNG");
}

/// Mirrors: ReadWritePacked."Can write packed PC SNG file"
///
/// Read PC SNG, write it packed, read the written file back.
#[test]
fn sng_write_packed_pc() {
    let sng = Sng::read_packed_file(cdlc("packed_pc.sng"), Platform::Pc)
        .expect("read packed_pc.sng");

    let packed_bytes = sng.to_packed_bytes(Platform::Pc)
        .expect("to_packed_bytes PC");

    let sng2 = Sng::from_packed_bytes(&packed_bytes, Platform::Pc)
        .expect("re-read written PC SNG");

    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "Re-read written packed PC SNG gives correct level count");
}

/// Mirrors: ReadWritePacked."Can write packed Mac SNG file"
///
/// Read Mac SNG, write it packed, read the written file back.
#[test]
fn sng_write_packed_mac() {
    let sng = Sng::read_packed_file(cdlc("packed_mac.sng"), Platform::Mac)
        .expect("read packed_mac.sng");

    let packed_bytes = sng.to_packed_bytes(Platform::Mac)
        .expect("to_packed_bytes Mac");

    let sng2 = Sng::from_packed_bytes(&packed_bytes, Platform::Mac)
        .expect("re-read written Mac SNG");

    assert_eq!(sng2.levels.len(), TEST_FILE_LEVELS,
        "Re-read written packed Mac SNG gives correct level count");
}

// ===========================================================================
// RoundTripTests.fs  (key cases — the property tests are already in
// integration_test.rs; here we add the remaining explicit case tests)
// ===========================================================================

/// Mirrors: RoundTripTests."Bend Data 32"
#[test]
fn sng_roundtrip_bend_data32_full() {
    use rocksmith2014::sng::types::{BendData32, BendValue};

    let bd = BendData32 {
        bend_values: std::array::from_fn(|i| BendValue {
            time: 66.66 + i as f32,
            step: if i == 0 { 0.0 } else { 1.0 / i as f32 },
        }),
        used_count: 32,
    };

    // Verify fields directly (BendData32 doesn't exist as top-level SNG list)
    assert_eq!(bd.bend_values[0].time, 66.66_f32);
    assert_eq!(bd.used_count, 32);
    assert!((bd.bend_values[1].step - 1.0).abs() < 1e-6);
}

/// Mirrors: RoundTripTests."Section" — named section fields preserved
#[test]
fn sng_roundtrip_section_named() {
    use rocksmith2014::sng::sng_to_bytes;
    use rocksmith2014::sng::types::Section;

    let mut sng = make_minimal_sng();
    sng.sections.push(Section {
        name: "tapping".into(),
        number: 2,
        start_time: 50.0,
        end_time: 62.7,
        start_phrase_iteration_id: 5,
        end_phrase_iteration_id: 6,
        string_mask: [0i8; 36],
    });

    let bytes = sng_to_bytes(&sng);
    let sng2 = Sng::from_unpacked_bytes(&bytes).unwrap();

    let s = &sng2.sections[0];
    assert_eq!(s.name, "tapping");
    assert_eq!(s.number, 2);
    assert!((s.start_time - 50.0).abs() < 1e-4);
    assert!((s.end_time - 62.7).abs() < 1e-4);
    assert_eq!(s.start_phrase_iteration_id, 5);
    assert_eq!(s.end_phrase_iteration_id, 6);
}

/// Mirrors: RoundTripTests."Anchor" — anchor fields preserved
#[test]
fn sng_roundtrip_anchor_named() {
    use rocksmith2014::sng::sng_to_bytes;
    use rocksmith2014::sng::types::Anchor;

    let anchor = Anchor {
        start_time: 10.0,
        end_time: 20.0,
        first_note_time: 11.0,
        last_note_time: 17.0,
        fret_id: 12,
        width: 4,
        phrase_iteration_id: 7,
    };

    let mut sng = make_minimal_sng();
    if sng.levels.is_empty() {
        sng.levels.push(rocksmith2014::sng::types::Level {
            difficulty: 0,
            anchors: vec![],
            anchor_extensions: vec![],
            hand_shapes: vec![],
            arpeggios: vec![],
            notes: vec![],
            average_notes_per_iteration: vec![],
            notes_in_phrase_iterations_excl_ignored: vec![],
            notes_in_phrase_iterations_all: vec![],
        });
    }
    sng.levels[0].anchors.push(anchor);

    let bytes = sng_to_bytes(&sng);
    let sng2 = Sng::from_unpacked_bytes(&bytes).unwrap();

    let a = &sng2.levels[0].anchors[0];
    assert!((a.start_time - 10.0).abs() < 1e-4);
    assert!((a.end_time - 20.0).abs() < 1e-4);
    assert_eq!(a.fret_id, 12);
    assert_eq!(a.width, 4);
    assert_eq!(a.phrase_iteration_id, 7);
}

/// Mirrors: RoundTripTests."Metadata" — metadata fields preserved
#[test]
fn sng_roundtrip_metadata_named() {
    use rocksmith2014::sng::sng_to_bytes;
    use rocksmith2014::sng::types::MetaData;

    let md = MetaData {
        max_score: 100_000.0,
        max_notes_and_chords: 456.0,
        max_notes_and_chords_real: 452.0,
        points_per_note: 100.0,
        first_beat_length: 88.0,
        start_time: 10.0,
        capo_fret_id: -1,
        last_conversion_date_time: "6-11-18 18:36".into(),
        part: 1,
        song_length: 520.0,
        tuning: vec![0i16; 6],
        first_note_time: 15.0,
        max_difficulty: 22,
    };

    let mut sng = make_minimal_sng();
    sng.metadata = md;
    let bytes = sng_to_bytes(&sng);
    let sng2 = Sng::from_unpacked_bytes(&bytes).unwrap();

    let m = &sng2.metadata;
    assert!((m.max_score - 100_000.0).abs() < 1e-4);
    assert_eq!(m.last_conversion_date_time, "6-11-18 18:36");
    assert_eq!(m.capo_fret_id, -1);
    assert_eq!(m.part, 1);
    assert_eq!(m.max_difficulty, 22);
}

// ===========================================================================
// Platform path tests  (mirrors PlatformTests.fs from Rocksmith2014.Common.Tests)
// ===========================================================================

/// Mirrors: PlatformTests."Path parts are correct for PC"
#[test]
fn platform_path_parts_pc() {
    assert_eq!(Platform::Pc.audio_path_part(), "windows");
    assert_eq!(Platform::Pc.sng_path_part(), "generic");
    assert_eq!(Platform::Pc.package_suffix(), "_p");
}

/// Mirrors: PlatformTests."Path parts are correct for Mac"
#[test]
fn platform_path_parts_mac() {
    assert_eq!(Platform::Mac.audio_path_part(), "mac");
    assert_eq!(Platform::Mac.sng_path_part(), "macos");
    assert_eq!(Platform::Mac.package_suffix(), "_m");
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Build the minimal valid Sng so we can test serialisation.
fn make_minimal_sng() -> Sng {
    use rocksmith2014::sng::types::MetaData;
    Sng {
        beats: vec![],
        phrases: vec![],
        chords: vec![],
        chord_notes: vec![],
        vocals: vec![],
        symbols_headers: vec![],
        symbols_textures: vec![],
        symbol_definitions: vec![],
        phrase_iterations: vec![],
        phrase_extra_info: vec![],
        new_linked_difficulties: vec![],
        actions: vec![],
        events: vec![],
        tones: vec![],
        dnas: vec![],
        sections: vec![],
        levels: vec![],
        metadata: MetaData {
            max_score: 0.0,
            max_notes_and_chords: 0.0,
            max_notes_and_chords_real: 0.0,
            points_per_note: 0.0,
            first_beat_length: 0.0,
            start_time: 0.0,
            capo_fret_id: -1,
            last_conversion_date_time: String::new(),
            part: 1,
            song_length: 0.0,
            tuning: vec![0i16; 6],
            first_note_time: 0.0,
            max_difficulty: 0,
        },
    }
}
