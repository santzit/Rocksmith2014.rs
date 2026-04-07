//! Integration tests that load real XML files copied from Rocksmith2014.NET.
//!
//! Mirrors the .NET test suite:
//! - `InstrumentalArrangementTests.cs` — loading and manipulating arrangements
//! - `MetaDataTests.cs`               — metadata field values
//! - `AnchorTests.cs`                 — anchor struct equality
//! - `NoteTests.cs`                   — note mask flag accessors (getter/setter)
//! - `ChordTests.cs`                  — chord mask flag accessors

use rocksmith2014_xml::*;
use std::path::PathBuf;

/// Returns the path to the XML test data directory.
fn xml_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("Rocksmith2014.XML.Tests")
}

// ---------------------------------------------------------------------------
// InstrumentalArrangementTests — loading from file
// ---------------------------------------------------------------------------

/// "Can be loaded from an XML file"
///
/// Mirrors `InstrumentalArrangementTests.CanRemoveDD`: load instrumental.xml
/// and confirm it has multiple difficulty levels (the arrangement has DD).
#[test]
fn test_can_load_instrumental_xml() {
    let arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");
    assert!(
        arr.levels.len() > 1,
        "instrumental.xml should have multiple DD levels, got {}",
        arr.levels.len()
    );
}

/// Round-trip: load → serialise → reload and confirm the arrangement is stable.
#[test]
fn test_instrumental_xml_roundtrip() {
    let arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");
    let xml_str = arr.to_xml().expect("to_xml");
    let arr2 = InstrumentalArrangement::from_xml(&xml_str).expect("from_xml round-trip");

    assert_eq!(arr.levels.len(), arr2.levels.len(), "level count preserved");
    assert_eq!(arr.ebeats.len(), arr2.ebeats.len(), "ebeat count preserved");
    assert_eq!(
        arr.chord_templates.len(),
        arr2.chord_templates.len(),
        "chord template count preserved"
    );
    assert_eq!(
        arr.meta.song_name, arr2.meta.song_name,
        "song name preserved"
    );
}

// ---------------------------------------------------------------------------
// MetaDataTests — mirrors MetaDataTests.CanBeReadFromXMLFile
// ---------------------------------------------------------------------------

/// "Metadata can be read from XML file"
///
/// Mirrors `MetaDataTests.CanBeReadFromXMLFile`:
/// - `Title` == "Test Instrumental"
/// - `AverageTempo` ≈ 160.541
/// - `ArtistNameSort` == "Test"
/// - `LastConversionDateTime` == "5-17-20 15:21"
#[test]
fn test_metadata_read_from_xml_file() {
    let arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");

    assert_eq!(arr.meta.song_name, "Test Instrumental");
    assert!(
        (arr.meta.average_tempo - 160.541).abs() < 0.001,
        "average_tempo should be ~160.541, got {}",
        arr.meta.average_tempo
    );
    assert_eq!(arr.meta.artist_name_sort, "Test");
    assert_eq!(arr.meta.last_conversion_date_time, "5-17-20 15:21");
}

// ---------------------------------------------------------------------------
// AnchorTests — mirrors AnchorTests (structural equality / copy)
// ---------------------------------------------------------------------------

/// "Anchor copy has all the same values"
///
/// Mirrors `AnchorTests.CopyConstructorCopiesAllValues` and
/// `AnchorTests.UsesStructuralEquality`.
#[test]
fn test_anchor_equality() {
    let a1 = Anchor {
        time: 4567,
        fret: 22,
        width: 6,
        end_time: 0,
    };
    let a2 = a1.clone();

    assert_eq!(a1.fret, a2.fret);
    assert_eq!(a1.time, a2.time);
    assert_eq!(a1.width, a2.width);
}

// ---------------------------------------------------------------------------
// NoteTests — mirrors NoteMaskAccessPropertiesGettersTest / SettersTest
// ---------------------------------------------------------------------------

/// "NoteMask flag bits can be set and cleared independently"
///
/// Mirrors `NoteTests.NoteMaskAccessPropertiesSettersTest`.
#[test]
fn test_note_mask_setters() {
    let base = NoteMask::LINK_NEXT | NoteMask::PALM_MUTE;

    let with_accent = base | NoteMask::ACCENT;
    assert!(with_accent.contains(NoteMask::ACCENT));
    let without_accent = with_accent & !NoteMask::ACCENT;
    assert!(!without_accent.contains(NoteMask::ACCENT));
    assert!(without_accent.contains(NoteMask::LINK_NEXT));

    let with_ho = base | NoteMask::HAMMER_ON;
    assert!(with_ho.contains(NoteMask::HAMMER_ON));
    let without_ho = with_ho & !NoteMask::HAMMER_ON;
    assert!(!without_ho.contains(NoteMask::HAMMER_ON));

    let with_harm = base | NoteMask::HARMONIC;
    assert!(with_harm.contains(NoteMask::HARMONIC));
    let without_harm = with_harm & !NoteMask::HARMONIC;
    assert!(!without_harm.contains(NoteMask::HARMONIC));
}

/// "NoteMask flags read back correctly"
///
/// Mirrors `NoteTests.NoteMaskAccessPropertiesGettersTest`.
#[test]
fn test_note_mask_getters() {
    let empty = NoteMask::empty();
    assert!(!empty.contains(NoteMask::ACCENT));
    assert!(!empty.contains(NoteMask::HAMMER_ON));
    assert!(!empty.contains(NoteMask::HARMONIC));
    assert!(!empty.contains(NoteMask::PINCH_HARMONIC));
    assert!(!empty.contains(NoteMask::IGNORE));
    assert!(!empty.contains(NoteMask::LINK_NEXT));
    assert!(!empty.contains(NoteMask::FRET_HAND_MUTE));
    assert!(!empty.contains(NoteMask::PALM_MUTE));
    assert!(!empty.contains(NoteMask::PULL_OFF));
    assert!(!empty.contains(NoteMask::TREMOLO));

    let set = NoteMask::ACCENT
        | NoteMask::HAMMER_ON
        | NoteMask::HARMONIC
        | NoteMask::PINCH_HARMONIC
        | NoteMask::IGNORE;
    assert!(set.contains(NoteMask::ACCENT));
    assert!(set.contains(NoteMask::HAMMER_ON));
    assert!(set.contains(NoteMask::HARMONIC));
    assert!(set.contains(NoteMask::PINCH_HARMONIC));
    assert!(set.contains(NoteMask::IGNORE));
    assert!(!set.contains(NoteMask::LINK_NEXT));
    assert!(!set.contains(NoteMask::PALM_MUTE));

    let set2 = NoteMask::LINK_NEXT
        | NoteMask::FRET_HAND_MUTE
        | NoteMask::PALM_MUTE
        | NoteMask::PULL_OFF
        | NoteMask::TREMOLO;
    assert!(!set2.contains(NoteMask::ACCENT));
    assert!(set2.contains(NoteMask::LINK_NEXT));
    assert!(set2.contains(NoteMask::FRET_HAND_MUTE));
    assert!(set2.contains(NoteMask::PALM_MUTE));
    assert!(set2.contains(NoteMask::PULL_OFF));
    assert!(set2.contains(NoteMask::TREMOLO));
}

// ---------------------------------------------------------------------------
// ChordTests — mirrors ChordMaskAccessPropertiesGettersTest / SettersTest
// ---------------------------------------------------------------------------

/// "ChordMask flag bits can be set and cleared independently"
///
/// Mirrors `ChordTests.ChordMaskAccessProperiesSettersTest`.
#[test]
fn test_chord_mask_setters() {
    let base = ChordMask::HIGH_DENSITY;

    let with_accent = base | ChordMask::ACCENT;
    assert!(with_accent.contains(ChordMask::ACCENT));
    let without_accent = with_accent & !ChordMask::ACCENT;
    assert!(!without_accent.contains(ChordMask::ACCENT));

    let with_fhm = ChordMask::empty() | ChordMask::FRET_HAND_MUTE;
    assert!(with_fhm.contains(ChordMask::FRET_HAND_MUTE));
    let without_fhm = with_fhm & !ChordMask::FRET_HAND_MUTE;
    assert!(!without_fhm.contains(ChordMask::FRET_HAND_MUTE));

    let base2 = ChordMask::FRET_HAND_MUTE | ChordMask::ACCENT;
    let with_hd = base2 | ChordMask::HIGH_DENSITY;
    assert!(with_hd.contains(ChordMask::HIGH_DENSITY));
    let without_hd = with_hd & !ChordMask::HIGH_DENSITY;
    assert!(!without_hd.contains(ChordMask::HIGH_DENSITY));

    let with_hopo = base2 | ChordMask::HOPO;
    assert!(with_hopo.contains(ChordMask::HOPO));
    let without_hopo = with_hopo & !ChordMask::HOPO;
    assert!(!without_hopo.contains(ChordMask::HOPO));

    let with_ig = base2 | ChordMask::HIGH_DENSITY | ChordMask::IGNORE;
    assert!(with_ig.contains(ChordMask::IGNORE));
    let without_ig = with_ig & !ChordMask::IGNORE;
    assert!(!without_ig.contains(ChordMask::IGNORE));

    let with_ln = with_ig | ChordMask::LINK_NEXT;
    assert!(with_ln.contains(ChordMask::LINK_NEXT));
    let without_ln = with_ln & !ChordMask::LINK_NEXT;
    assert!(!without_ln.contains(ChordMask::LINK_NEXT));

    let with_pm = ChordMask::PALM_MUTE;
    assert!(with_pm.contains(ChordMask::PALM_MUTE));
    let without_pm = with_pm & !ChordMask::PALM_MUTE;
    assert!(!without_pm.contains(ChordMask::PALM_MUTE));
}

/// "ChordMask flags read back correctly"
///
/// Mirrors `ChordTests.ChordMaskAccessProperiesGettersTest`.
#[test]
fn test_chord_mask_getters() {
    let empty = ChordMask::empty();
    assert!(!empty.contains(ChordMask::ACCENT));
    assert!(!empty.contains(ChordMask::FRET_HAND_MUTE));
    assert!(!empty.contains(ChordMask::HIGH_DENSITY));
    assert!(!empty.contains(ChordMask::HOPO));
    assert!(!empty.contains(ChordMask::IGNORE));
    assert!(!empty.contains(ChordMask::LINK_NEXT));
    assert!(!empty.contains(ChordMask::PALM_MUTE));

    let set = ChordMask::ACCENT | ChordMask::FRET_HAND_MUTE | ChordMask::HIGH_DENSITY;
    assert!(set.contains(ChordMask::ACCENT));
    assert!(set.contains(ChordMask::FRET_HAND_MUTE));
    assert!(set.contains(ChordMask::HIGH_DENSITY));
    assert!(!set.contains(ChordMask::HOPO));
    assert!(!set.contains(ChordMask::IGNORE));
    assert!(!set.contains(ChordMask::LINK_NEXT));
    assert!(!set.contains(ChordMask::PALM_MUTE));

    let set2 = ChordMask::HOPO | ChordMask::IGNORE | ChordMask::LINK_NEXT;
    assert!(!set2.contains(ChordMask::ACCENT));
    assert!(set2.contains(ChordMask::HOPO));
    assert!(set2.contains(ChordMask::IGNORE));
    assert!(set2.contains(ChordMask::LINK_NEXT));
    assert!(!set2.contains(ChordMask::PALM_MUTE));

    let set3 = ChordMask::PALM_MUTE;
    assert!(!set3.contains(ChordMask::ACCENT));
    assert!(set3.contains(ChordMask::PALM_MUTE));
}

/// "ChordMask HOPO round-trips through XML serialization"
#[test]
fn test_chord_mask_hopo_roundtrip() {
    let mut arr = InstrumentalArrangement::default();
    arr.levels.push(Level {
        difficulty: 0,
        chords: vec![Chord {
            time: 1000,
            chord_id: 0,
            mask: ChordMask::HOPO,
            ..Default::default()
        }],
        ..Default::default()
    });

    let xml = arr.to_xml().expect("to_xml");
    let arr2 = InstrumentalArrangement::from_xml(&xml).expect("from_xml");
    assert!(
        arr2.levels[0].chords[0].mask.contains(ChordMask::HOPO),
        "HOPO flag should survive XML round-trip"
    );
    assert!(
        !arr2.levels[0].chords[0].mask.contains(ChordMask::ACCENT),
        "ACCENT flag should not be set"
    );
}
