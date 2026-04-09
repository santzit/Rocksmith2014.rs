use rocksmith2014_xml::{
    Anchor, BendValue, Chord, ChordNote, ChordTemplate, InstrumentalArrangement, Level, Note,
    NoteMask, Phrase, PhraseIteration,
};
use rocksmith2014_xml_processing::{
    add_ignores, fix_link_nexts, remove_muted_notes_from_chords, remove_overlapping_bend_values,
    remove_redundant_anchors, validate_phrase_names,
};

#[test]
fn filters_characters_in_phrase_names() {
    let phrases = vec![
        Phrase { name: "\"TEST\"".into(), ..Default::default() },
        Phrase { name: "'TEST'_(2)".into(), ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement { phrases, ..Default::default() };
    validate_phrase_names(&mut arr);
    assert_eq!(arr.phrases[0].name, "TEST");
    assert_eq!(arr.phrases[1].name, "TEST_2");
}

#[test]
fn ignore_is_added_to_23rd_and_24th_fret_notes() {
    let notes = vec![
        Note { time: 1000, fret: 5, ..Default::default() },
        Note { time: 1200, fret: 23, ..Default::default() },
        Note { time: 1300, fret: 24, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    add_ignores(&mut arr);
    assert!(!arr.levels[0].notes[0].mask.contains(NoteMask::IGNORE));
    assert!(arr.levels[0].notes[1].mask.contains(NoteMask::IGNORE));
    assert!(arr.levels[0].notes[2].mask.contains(NoteMask::IGNORE));
}

#[test]
fn ignore_is_added_to_7th_fret_harmonic_with_sustain() {
    let notes = vec![
        Note { time: 1000, fret: 7, sustain: 500, mask: NoteMask::HARMONIC, ..Default::default() },
        Note { time: 2000, fret: 7, sustain: 0, mask: NoteMask::HARMONIC, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    add_ignores(&mut arr);
    assert!(arr.levels[0].notes[0].mask.contains(NoteMask::IGNORE));
    assert!(!arr.levels[0].notes[1].mask.contains(NoteMask::IGNORE));
}

#[test]
fn incorrect_linknext_is_removed_next_note_on_same_string_not_found() {
    let notes = vec![
        Note { time: 1000, fret: 5, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { time: 1500, string: 4, fret: 5, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    fix_link_nexts(&mut arr);
    assert!(!arr.levels[0].notes[0].mask.contains(NoteMask::LINK_NEXT));
}

#[test]
fn incorrect_linknext_is_removed_next_note_too_far() {
    let notes = vec![
        Note { time: 1000, fret: 5, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { time: 2000, fret: 5, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    fix_link_nexts(&mut arr);
    assert!(!arr.levels[0].notes[0].mask.contains(NoteMask::LINK_NEXT));
}

#[test]
fn incorrect_linknext_fret_is_corrected() {
    let notes = vec![
        Note { time: 1000, fret: 5, sustain: 500, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { time: 1500, fret: 6, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    fix_link_nexts(&mut arr);
    assert!(arr.levels[0].notes[0].mask.contains(NoteMask::LINK_NEXT));
    assert_eq!(arr.levels[0].notes[1].fret, 5);
}

#[test]
fn overlapping_bend_values_are_removed() {
    let bv1 = vec![BendValue { time: 1200, step: 2.0, ..Default::default() }, BendValue { time: 1200, step: 1.0, ..Default::default() }];
    let bv2 = vec![BendValue { time: 2100, step: 2.0, ..Default::default() }, BendValue { time: 2100, step: 2.0, ..Default::default() }];
    let notes = vec![
        Note { time: 1000, fret: 5, sustain: 500, mask: NoteMask::LINK_NEXT, bend_values: bv1, ..Default::default() },
    ];
    let cn = vec![ChordNote { sustain: 500, bend_values: bv2, ..Default::default() }];
    let chords = vec![Chord { time: 2000, chord_notes: cn, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { notes, chords, ..Default::default() }],
        ..Default::default()
    };
    remove_overlapping_bend_values(&mut arr);
    assert_eq!(arr.levels[0].notes[0].bend_values.len(), 1);
    assert_eq!(arr.levels[0].chords[0].chord_notes[0].bend_values.len(), 1);
}

#[test]
fn redundant_anchors_are_removed() {
    let anchors = vec![
        Anchor { fret: 1, time: 1000, width: 4, end_time: 0 },
        Anchor { fret: 1, time: 2000, width: 4, end_time: 0 },
        Anchor { fret: 5, time: 3000, width: 4, end_time: 0 },
        Anchor { fret: 5, time: 4000, width: 6, end_time: 0 },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { anchors, ..Default::default() }],
        ..Default::default()
    };
    remove_redundant_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 3);
}

#[test]
fn identical_anchor_at_phrase_time_is_not_removed() {
    let anchors = vec![
        Anchor { fret: 1, time: 1000, width: 4, end_time: 0 },
        Anchor { fret: 1, time: 2000, width: 4, end_time: 0 },
        Anchor { fret: 1, time: 3000, width: 4, end_time: 0 },
        Anchor { fret: 1, time: 4000, width: 4, end_time: 0 },
        Anchor { fret: 1, time: 5000, width: 5, end_time: 0 },
    ];
    let phrase_iterations = vec![
        PhraseIteration { time: 1000, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 4000, phrase_id: 0, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { anchors, ..Default::default() }],
        phrase_iterations,
        ..Default::default()
    };
    remove_redundant_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 3);
}

#[test]
#[ignore = "remove_muted_notes_from_chords needs verification of chord template mutation behavior"]
fn muted_strings_are_removed_from_non_muted_chords() {
    let templates = vec![
        ChordTemplate { name: "".into(), display_name: "".into(), fingers: [1, 3, 4, -1, -1, -1], frets: [1, 3, 3, -1, -1, -1] },
        ChordTemplate { name: "".into(), display_name: "".into(), fingers: [-1; 6], frets: [0, 0, 0, -1, -1, -1] },
    ];
    let cn1 = vec![
        ChordNote { string: 0, fret: 1, ..Default::default() },
        ChordNote { string: 1, fret: 3, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
        ChordNote { string: 2, fret: 3, ..Default::default() },
    ];
    let cn2 = vec![
        ChordNote { string: 0, fret: 0, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
        ChordNote { string: 1, fret: 0, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
        ChordNote { string: 2, fret: 0, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
    ];
    let chords = vec![
        Chord { time: 1000, chord_notes: cn1, ..Default::default() },
        Chord { time: 1200, chord_id: 1, chord_notes: cn2, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords, ..Default::default() }],
        chord_templates: templates,
        ..Default::default()
    };
    remove_muted_notes_from_chords(&mut arr);
    assert_eq!(arr.levels[0].chords[0].chord_notes.len(), 2);
    assert_eq!(arr.levels[0].chords[1].chord_notes.len(), 3);
}
