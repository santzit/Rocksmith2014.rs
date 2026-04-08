//! Tests for the [`Chord`] type and [`ChordMask`] bitflags.
//!
//! Mirrors `ChordTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::{Chord, ChordMask, ChordNote, InstrumentalArrangement, Level};

/// Mirrors `ChordTests.ChordMaskAccessProperiesSettersTest`
#[test]
fn chord_mask_access_properties_setters_test() {
    let mut mask = ChordMask::HIGH_DENSITY;

    mask |= ChordMask::ACCENT;
    assert!(mask.contains(ChordMask::ACCENT));
    mask &= !ChordMask::ACCENT;
    assert!(!mask.contains(ChordMask::ACCENT));

    mask = ChordMask::empty();
    mask |= ChordMask::FRET_HAND_MUTE;
    assert!(mask.contains(ChordMask::FRET_HAND_MUTE));
    mask &= !ChordMask::FRET_HAND_MUTE;
    assert!(!mask.contains(ChordMask::FRET_HAND_MUTE));

    mask = ChordMask::FRET_HAND_MUTE | ChordMask::ACCENT;
    mask |= ChordMask::HIGH_DENSITY;
    assert!(mask.contains(ChordMask::HIGH_DENSITY));
    mask &= !ChordMask::HIGH_DENSITY;
    assert!(!mask.contains(ChordMask::HIGH_DENSITY));

    mask |= ChordMask::HOPO;
    assert!(mask.contains(ChordMask::HOPO));
    mask &= !ChordMask::HOPO;
    assert!(!mask.contains(ChordMask::HOPO));

    mask |= ChordMask::IGNORE;
    assert!(mask.contains(ChordMask::IGNORE));
    mask &= !ChordMask::IGNORE;
    assert!(!mask.contains(ChordMask::IGNORE));

    mask |= ChordMask::LINK_NEXT;
    assert!(mask.contains(ChordMask::LINK_NEXT));
    mask &= !ChordMask::LINK_NEXT;
    assert!(!mask.contains(ChordMask::LINK_NEXT));

    mask = ChordMask::PALM_MUTE;
    assert!(mask.contains(ChordMask::PALM_MUTE));
    mask &= !ChordMask::PALM_MUTE;
    assert!(!mask.contains(ChordMask::PALM_MUTE));
}

/// Mirrors `ChordTests.ChordMaskAccessProperiesGettersTest`
#[test]
fn chord_mask_access_properties_getters_test() {
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

/// Mirrors `ChordTests.CopyConstructorCopiesAllValues`
#[test]
fn copy_constructor_copies_all_values() {
    let chord1 = Chord {
        time: 5000,
        chord_id: 77,
        mask: ChordMask::ACCENT | ChordMask::IGNORE,
        chord_notes: vec![
            ChordNote {
                string: 0,
                fret: 5,
                ..Default::default()
            },
            ChordNote {
                string: 1,
                fret: 7,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let chord2 = chord1.clone();

    assert_eq!(chord1.time, chord2.time);
    assert_eq!(chord1.chord_id, chord2.chord_id);
    assert_eq!(chord1.mask, chord2.mask);
    assert_eq!(chord1.chord_notes.len(), chord2.chord_notes.len());
}

/// Mirrors `ChordTests.HasChordNotesReturnsCorrectValue`
#[test]
fn has_chord_notes_returns_correct_value() {
    let chord = Chord {
        chord_notes: vec![
            ChordNote {
                string: 0,
                fret: 5,
                ..Default::default()
            },
            ChordNote {
                string: 1,
                fret: 7,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert!(!chord.chord_notes.is_empty());

    let empty_chord = Chord::default();
    assert!(empty_chord.chord_notes.is_empty());
}

/// ChordMask HOPO round-trips through XML serialization.
#[test]
fn chord_mask_hopo_roundtrip() {
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
    assert!(arr2.levels[0].chords[0].mask.contains(ChordMask::HOPO));
    assert!(!arr2.levels[0].chords[0].mask.contains(ChordMask::ACCENT));
}
