use rocksmith2014_xml::{
    BendValue, Chord, ChordMask, ChordNote, ChordTemplate, InstrumentalArrangement, Level,
    MetaData, Note, NoteMask, Phrase, PhraseIteration, Section, ToneChange,
};
use rocksmith2014_xml_processing::checkers::checker::check_chords;
use rocksmith2014_xml_processing::issue::IssueType;

fn test_arr() -> InstrumentalArrangement {
    InstrumentalArrangement {
        sections: vec![
            Section { name: "noguitar".into(), start_time: 6000, number: 1, end_time: 0 },
            Section { name: "riff".into(), start_time: 6500, number: 1, end_time: 0 },
            Section { name: "noguitar".into(), start_time: 8000, number: 2, end_time: 0 },
        ],
        chord_templates: vec![
            ChordTemplate { name: "".into(), display_name: "".into(), fingers: [2, 2, -1, -1, -1, -1], frets: [2, 2, -1, -1, -1, -1] },
            ChordTemplate { name: "WEIRDO1".into(), display_name: "".into(), fingers: [-1, 2, 3, 1, -1, -1], frets: [-1, 2, 1, 2, -1, -1] },
            ChordTemplate { name: "WEIRDO2".into(), display_name: "".into(), fingers: [-1, 2, 4, 3, -1, -1], frets: [-1, 2, 1, 2, -1, -1] },
            ChordTemplate { name: "THUMB".into(), display_name: "".into(), fingers: [0, -1, 1, 1, 1, -1], frets: [2, -1, 1, 1, 1, -1] },
            ChordTemplate { name: "BARRE2".into(), display_name: "".into(), fingers: [1, -1, -1, -1, 1, -1], frets: [3, -1, 0, 0, 3, -1] },
            ChordTemplate { name: "BARRE2".into(), display_name: "".into(), fingers: [2, -1, 2, 2, 1, -1], frets: [2, 0, 2, 2, 2, -1] },
            ChordTemplate { name: "THUMB2".into(), display_name: "".into(), fingers: [0, -1, -1, -1, -1, -1], frets: [5, -1, 0, 0, 0, -1] },
            ChordTemplate { name: "THUMB3".into(), display_name: "".into(), fingers: [0, 3, 4, 2, -1, -1], frets: [1, 3, 3, 2, -1, -1] },
        ],
        phrases: vec![Phrase { name: "mover6.700".into(), max_difficulty: 0, ..Default::default() }],
        phrase_iterations: vec![PhraseIteration { time: 6500, phrase_id: 0, ..Default::default() }],
        tones: vec![ToneChange { name: "test".into(), time: 5555, id: 1 }],
        meta: MetaData { song_length: 10000, ..Default::default() },
        ..Default::default()
    }
}

#[test]
fn detects_chord_note_with_linknext_and_unpitched_slide() {
    let cn = vec![ChordNote { slide_unpitch_to: 10, sustain: 100, mask: NoteMask::LINK_NEXT, ..Default::default() }];
    let chords = vec![Chord { chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| matches!(r.issue_type(), IssueType::UnpitchedSlideWithLinkNext)));
    assert!(results.iter().any(|r| matches!(r.issue_type(), IssueType::LinkNextMissingTargetNote)));
}

#[test]
fn detects_chord_note_with_both_harmonic_and_pinch_harmonic() {
    let cn = vec![ChordNote { mask: NoteMask::HARMONIC | NoteMask::PINCH_HARMONIC, ..Default::default() }];
    let chords = vec![Chord { chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::DoubleHarmonic));
}

#[test]
fn detects_harmonic_chord_note_on_7th_fret_with_sustain() {
    let cn = vec![ChordNote { fret: 7, sustain: 200, mask: NoteMask::HARMONIC, ..Default::default() }];
    let chords = vec![Chord { chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::SeventhFretHarmonicWithSustain));
}

#[test]
fn detects_tone_change_that_occurs_on_a_chord() {
    let cn = vec![ChordNote::default()];
    let chords = vec![Chord { time: 5555, chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::ToneChangeOnNote));
}

#[test]
fn detects_chord_inside_noguitar_section() {
    let chords = vec![Chord { time: 6100, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteInsideNoguitarSection));
}

#[test]
fn detects_chord_note_linknext_slide_fret_mismatch() {
    let cn = vec![ChordNote { sustain: 100, slide_to: 3, fret: 1, mask: NoteMask::LINK_NEXT, ..Default::default() }];
    let chords = vec![Chord { time: 1000, chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let notes = vec![Note { time: 1100, fret: 12, ..Default::default() }];
    let level = Level { chords, notes, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextSlideMismatch));
}

#[test]
fn detects_chord_note_linknext_bend_value_mismatch() {
    let bv = vec![BendValue { time: 1050, step: 1.0, ..Default::default() }];
    let cn = vec![ChordNote { sustain: 100, fret: 1, bend_values: bv, mask: NoteMask::LINK_NEXT, ..Default::default() }];
    let chords = vec![Chord { time: 1000, chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let notes = vec![Note { time: 1100, fret: 1, ..Default::default() }];
    let level = Level { chords, notes, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextBendMismatch));
}

#[test]
fn detects_missing_bend_value_on_chord_note() {
    let bend_values = vec![BendValue { time: 0, step: 0.0, ..Default::default() }];
    let cn = vec![
        ChordNote { string: 1, sustain: 100, bend_values, ..Default::default() },
        ChordNote { string: 2, sustain: 100, ..Default::default() },
    ];
    let chords = vec![Chord { time: 1000, chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::MissingBendValue));
}

#[test]
fn detects_linknext_chord_without_any_chord_notes() {
    let chords = vec![Chord { time: 1000, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::MissingLinkNextChordNotes));
}

#[test]
fn detects_linknext_chord_without_linknext_chord_notes() {
    let cn = vec![
        ChordNote { string: 1, sustain: 100, ..Default::default() },
        ChordNote { string: 2, sustain: 100, ..Default::default() },
    ];
    let chords = vec![Chord { time: 1000, chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::MissingLinkNextChordNotes));
}

#[test]
fn detects_chords_with_weird_fingering() {
    let cn = vec![ChordNote { string: 1, sustain: 100, ..Default::default() }];
    let chords = vec![
        Chord { chord_id: 1, chord_notes: cn.clone(), ..Default::default() },
        Chord { chord_id: 2, chord_notes: cn.clone(), ..Default::default() },
        Chord { chord_id: 3, chord_notes: cn.clone(), ..Default::default() },
    ];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| matches!(r.issue_type(), IssueType::PossiblyWrongChordFingering)));
}

#[test]
fn weird_fingering_check_ignores_chord_that_uses_thumb() {
    let cn = vec![ChordNote { string: 1, sustain: 100, ..Default::default() }];
    let chords = vec![Chord { chord_id: 7, chord_notes: cn, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert!(results.is_empty());
}

#[test]
fn detects_chords_with_barre_over_open_strings() {
    let cn = vec![ChordNote { string: 1, sustain: 100, ..Default::default() }];
    let chords = vec![
        Chord { chord_id: 4, chord_notes: cn.clone(), ..Default::default() },
        Chord { chord_id: 5, chord_notes: cn.clone(), ..Default::default() },
    ];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| matches!(r.issue_type(), IssueType::BarreOverOpenStrings)));
}

#[test]
fn detects_non_muted_chords_containing_muted_strings() {
    let cn1 = vec![
        ChordNote { string: 0, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
        ChordNote { string: 1, ..Default::default() },
    ];
    let cn2 = vec![
        ChordNote { string: 0, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
        ChordNote { string: 1, mask: NoteMask::FRET_HAND_MUTE, ..Default::default() },
    ];
    let chords = vec![
        Chord { chord_notes: cn1, ..Default::default() },
        Chord { chord_notes: cn2, ..Default::default() },
    ];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::MutedStringInNonMutedChord));
}

#[test]
fn overlapping_bend_values_are_detected_for_chords() {
    let cn1 = vec![
        ChordNote { string: 4, fret: 12, ..Default::default() },
        ChordNote {
            string: 5,
            fret: 12,
            sustain: 500,
            bend_values: vec![BendValue { time: 200, step: 2.0, ..Default::default() }, BendValue { time: 200, step: 1.0, ..Default::default() }],
            ..Default::default()
        },
    ];
    let chords = vec![Chord { time: 1000, chord_notes: cn1, ..Default::default() }];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::OverlappingBendValues));
}

#[test]
fn invalid_strings_on_bass_arrangement_are_detected_for_chords() {
    let cn1 = vec![
        ChordNote { string: 3, fret: 12, ..Default::default() },
        ChordNote { string: 4, fret: 12, ..Default::default() },
        ChordNote { string: 5, fret: 12, ..Default::default() },
    ];
    let chords = vec![Chord { time: 1000, chord_notes: cn1, ..Default::default() }];
    let level = Level { chords: chords.clone(), ..Default::default() };
    let mut arr = InstrumentalArrangement {
        levels: vec![level.clone()],
        meta: MetaData { song_length: 500_000, ..Default::default() },
        ..Default::default()
    };
    arr.meta.arrangement_properties.path_bass = 1;
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::InvalidBassArrangementString));
}

#[test]
fn detects_chord_after_end_phrase() {
    let chords = vec![Chord { time: 50_000, ..Default::default() }];
    let level = Level { chords: chords.clone(), ..Default::default() };
    let phrases = vec![
        Phrase { name: "Default".into(), ..Default::default() },
        Phrase { name: "end".into(), ..Default::default() },
    ];
    let phrase_iterations = vec![
        PhraseIteration { time: 1000, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 45_000, phrase_id: 1, ..Default::default() },
    ];
    let arr = InstrumentalArrangement {
        levels: vec![level.clone()],
        phrases,
        phrase_iterations,
        meta: MetaData { song_length: 500_000, ..Default::default() },
        ..Default::default()
    };
    let results = check_chords(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteAfterSongEnd));
    assert_eq!(results[0].time_code(), Some(50_000));
}

#[test]
fn detects_chords_with_techniques_that_require_sustain() {
    let cn1 = vec![ChordNote { fret: 1, slide_to: 2, sustain: 0, ..Default::default() }];
    let cn2 = vec![ChordNote { fret: 1, vibrato: 80, sustain: 1, ..Default::default() }];
    let cn3 = vec![ChordNote { fret: 1, sustain: 2, mask: NoteMask::TREMOLO, ..Default::default() }];
    let cn4 = vec![ChordNote { fret: 1, slide_unpitch_to: 7, sustain: 0, ..Default::default() }];
    let chords = vec![
        Chord { time: 1_000, chord_notes: cn1, ..Default::default() },
        Chord { time: 2_000, chord_notes: cn2, ..Default::default() },
        Chord { time: 3_000, chord_notes: cn3, ..Default::default() },
        Chord { time: 4_000, chord_notes: cn4, ..Default::default() },
    ];
    let level = Level { chords, ..Default::default() };
    let arr = test_arr();
    let results = check_chords(&arr, &level);
    assert!(!results.is_empty());
    assert!(results.iter().all(|r| matches!(r.issue_type(), IssueType::TechniqueNoteWithoutSustain)));
}
