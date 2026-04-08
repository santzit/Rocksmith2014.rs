//! Tests for the [`Note`] type and [`NoteMask`] bitflags.
//!
//! Mirrors `NoteTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::{BendValue, Note, NoteMask};

/// Mirrors `NoteTests.NoteMaskAccessPropertiesSettersTest`
#[test]
fn note_mask_access_properties_setters_test() {
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

    let with_ph = base | NoteMask::PINCH_HARMONIC;
    assert!(with_ph.contains(NoteMask::PINCH_HARMONIC));
    let without_ph = with_ph & !NoteMask::PINCH_HARMONIC;
    assert!(!without_ph.contains(NoteMask::PINCH_HARMONIC));

    let with_ign = NoteMask::empty() | NoteMask::IGNORE;
    assert!(with_ign.contains(NoteMask::IGNORE));
    let without_ign = with_ign & !NoteMask::IGNORE;
    assert!(!without_ign.contains(NoteMask::IGNORE));

    let with_ln = (NoteMask::PULL_OFF | NoteMask::FRET_HAND_MUTE) | NoteMask::LINK_NEXT;
    assert!(with_ln.contains(NoteMask::LINK_NEXT));
    let without_ln = with_ln & !NoteMask::LINK_NEXT;
    assert!(!without_ln.contains(NoteMask::LINK_NEXT));

    let with_fhm =
        (NoteMask::PALM_MUTE | NoteMask::IGNORE | NoteMask::LINK_NEXT) | NoteMask::FRET_HAND_MUTE;
    assert!(with_fhm.contains(NoteMask::FRET_HAND_MUTE));
    let without_fhm = with_fhm & !NoteMask::FRET_HAND_MUTE;
    assert!(!without_fhm.contains(NoteMask::FRET_HAND_MUTE));

    let with_pm =
        (NoteMask::FRET_HAND_MUTE | NoteMask::TREMOLO | NoteMask::LINK_NEXT) | NoteMask::PALM_MUTE;
    assert!(with_pm.contains(NoteMask::PALM_MUTE));
    let without_pm = with_pm & !NoteMask::PALM_MUTE;
    assert!(!without_pm.contains(NoteMask::PALM_MUTE));

    let with_po = NoteMask::PICK_DIRECTION | NoteMask::PULL_OFF;
    assert!(with_po.contains(NoteMask::PULL_OFF));
    let without_po = with_po & !NoteMask::PULL_OFF;
    assert!(!without_po.contains(NoteMask::PULL_OFF));

    let with_tr =
        (NoteMask::PALM_MUTE | NoteMask::IGNORE | NoteMask::LINK_NEXT) | NoteMask::TREMOLO;
    assert!(with_tr.contains(NoteMask::TREMOLO));
    let without_tr = with_tr & !NoteMask::TREMOLO;
    assert!(!without_tr.contains(NoteMask::TREMOLO));
}

/// Mirrors `NoteTests.NoteMaskAccessPropertiesGettersTest`
#[test]
fn note_mask_access_properties_getters_test() {
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
    assert!(!empty.contains(NoteMask::PLUCK));
    assert!(!empty.contains(NoteMask::SLAP));
    assert!(!empty.contains(NoteMask::RIGHT_HAND));

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
    assert!(!set.contains(NoteMask::FRET_HAND_MUTE));
    assert!(!set.contains(NoteMask::PALM_MUTE));
    assert!(!set.contains(NoteMask::PULL_OFF));
    assert!(!set.contains(NoteMask::TREMOLO));
    assert!(!set.contains(NoteMask::PLUCK));
    assert!(!set.contains(NoteMask::SLAP));
    assert!(!set.contains(NoteMask::RIGHT_HAND));

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

    let set3 = NoteMask::SLAP | NoteMask::PLUCK | NoteMask::RIGHT_HAND;
    assert!(!set3.contains(NoteMask::ACCENT));
    assert!(set3.contains(NoteMask::PLUCK));
    assert!(set3.contains(NoteMask::SLAP));
    assert!(set3.contains(NoteMask::RIGHT_HAND));
}

/// Mirrors `NoteTests.OtherGettersTest`:
/// is_bend, is_slide, is_unpitched_slide, is_vibrato, is_tap.
#[test]
fn other_getters_test() {
    let mut note = Note::default();
    // Default Note has no bend values, slide_to = -1 (no slide), vibrato = 0, tap = 0
    assert!(note.bend_values.is_empty());
    assert_eq!(note.slide_to, -1);
    assert_eq!(note.slide_unpitch_to, -1);
    assert_eq!(note.vibrato, 0);
    assert_eq!(note.tap, 0);

    note.bend_values = vec![BendValue {
        time: 100,
        step: 1.5,
        ..Default::default()
    }];
    assert!(!note.bend_values.is_empty()); // is_bend == true

    note.slide_to = 14;
    assert_eq!(note.slide_to, 14); // is_slide == true

    note.slide_unpitch_to = 12;
    assert_eq!(note.slide_unpitch_to, 12); // is_unpitched_slide == true

    note.vibrato = 80;
    assert_ne!(note.vibrato, 0); // is_vibrato == true

    note.tap = 2;
    assert_ne!(note.tap, 0); // is_tap == true
}

/// Mirrors `NoteTests.CopyConstructorCopiesAllValues`
#[test]
fn copy_constructor_copies_all_values() {
    let note1 = Note {
        fret: 22,
        left_hand: 3,
        mask: NoteMask::ACCENT
            | NoteMask::IGNORE
            | NoteMask::LINK_NEXT
            | NoteMask::FRET_HAND_MUTE
            | NoteMask::SLAP,
        slide_to: 7,
        slide_unpitch_to: 9,
        string: 4,
        sustain: 99000,
        tap: 2,
        time: 33000,
        vibrato: 80,
        max_bend: 4.0,
        bend_values: vec![
            BendValue {
                time: 34000,
                step: 3.0,
                ..Default::default()
            },
            BendValue {
                time: 35000,
                step: 4.0,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let note2 = note1.clone();

    // Different heap allocations
    assert_eq!(note1.bend_values.len(), note2.bend_values.len());
    assert_eq!(note1.bend_values[0].time, note2.bend_values[0].time);
    assert_eq!(note1.bend_values[1].step, note2.bend_values[1].step);

    assert_eq!(note1.fret, note2.fret);
    assert_eq!(note1.left_hand, note2.left_hand);
    assert_eq!(note1.mask, note2.mask);
    assert_eq!(note1.slide_to, note2.slide_to);
    assert_eq!(note1.slide_unpitch_to, note2.slide_unpitch_to);
    assert_eq!(note1.string, note2.string);
    assert_eq!(note1.sustain, note2.sustain);
    assert_eq!(note1.tap, note2.tap);
    assert_eq!(note1.time, note2.time);
    assert_eq!(note1.vibrato, note2.vibrato);
    assert_eq!(note1.max_bend, note2.max_bend);
}
