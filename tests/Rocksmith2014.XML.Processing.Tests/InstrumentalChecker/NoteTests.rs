use rocksmith2014_xml::{
    BendValue, Chord, ChordNote, ChordTemplate, InstrumentalArrangement, Level, MetaData,
    Note, NoteMask, Phrase, PhraseIteration, Section, ToneChange,
};
use rocksmith2014_xml_processing::checkers::checker::check_notes;
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
        ],
        phrases: vec![Phrase { name: "mover6.700".into(), max_difficulty: 0, ..Default::default() }],
        phrase_iterations: vec![PhraseIteration { time: 6500, phrase_id: 0, ..Default::default() }],
        tones: vec![ToneChange { name: "test".into(), time: 5555, id: 1 }],
        meta: MetaData { song_length: 10000, ..Default::default() },
        ..Default::default()
    }
}

#[test]
fn detects_unpitched_slide_note_with_linknext() {
    let notes = vec![
        Note { slide_unpitch_to: 12, sustain: 100, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 12, time: 100, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::UnpitchedSlideWithLinkNext));
}

#[test]
fn detects_note_with_both_harmonic_and_pinch_harmonic() {
    let notes = vec![
        Note { mask: NoteMask::HARMONIC | NoteMask::PINCH_HARMONIC, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::DoubleHarmonic));
}

#[test]
fn detects_harmonic_note_on_7th_fret_with_sustain() {
    let notes = vec![
        Note { fret: 7, sustain: 200, mask: NoteMask::HARMONIC, ..Default::default() },
        Note { fret: 7, mask: NoteMask::HARMONIC, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::SeventhFretHarmonicWithSustain));
}

#[test]
fn ignores_harmonic_note_on_7th_fret_with_sustain_when_ignore_set() {
    let notes = vec![
        Note { fret: 7, sustain: 200, mask: NoteMask::HARMONIC | NoteMask::IGNORE, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert!(results.is_empty());
}

#[test]
fn detects_note_with_missing_bend_values() {
    let notes = vec![
        Note { fret: 7, sustain: 100, bend_values: vec![BendValue { time: 0, step: 0.0, ..Default::default() }], ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::MissingBendValue));
}

#[test]
fn detects_tone_change_that_occurs_on_a_note() {
    let notes = vec![Note { time: 5555, ..Default::default() }];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::ToneChangeOnNote));
}

#[test]
fn detects_note_inside_noguitar_section() {
    let notes = vec![Note { time: 6000, ..Default::default() }];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteInsideNoguitarSection));
}

#[test]
fn detects_note_inside_last_noguitar_section() {
    let notes = vec![Note { time: 9000, ..Default::default() }];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteInsideNoguitarSection));
}

#[test]
fn detects_linknext_fret_mismatch() {
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 5, time: 1100, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextFretMismatch));
}

#[test]
fn detects_note_linked_to_a_chord() {
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, mask: NoteMask::LINK_NEXT, ..Default::default() },
    ];
    let chord_notes = vec![ChordNote { fret: 1, ..Default::default() }];
    let chords = vec![Chord { time: 1100, chord_notes, ..Default::default() }];
    let level = Level { notes, chords, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteLinkedToChord));
}

#[test]
fn detects_linknext_slide_fret_mismatch() {
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, slide_to: 4, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 5, time: 1100, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextSlideMismatch));
}

#[test]
fn detects_linknext_bend_value_mismatch_1() {
    let bv1 = vec![BendValue { time: 1050, step: 1.0, ..Default::default() }];
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, bend_values: bv1, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 1, time: 1100, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextBendMismatch));
}

#[test]
fn detects_linknext_bend_value_mismatch_2() {
    let bv1 = vec![BendValue { time: 1050, step: 1.0, ..Default::default() }];
    let bv2 = vec![BendValue { time: 1100, step: 2.0, ..Default::default() }];
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, bend_values: bv1, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 1, time: 1100, sustain: 100, bend_values: bv2, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::LinkNextBendMismatch));
}

#[test]
fn no_false_positive_when_no_bend_value_at_note_time() {
    let bv1 = vec![BendValue { time: 1000, step: 1.0, ..Default::default() }, BendValue { time: 1050, step: 0.0, ..Default::default() }];
    let bv2 = vec![BendValue { time: 1150, step: 1.0, ..Default::default() }];
    let notes = vec![
        Note { fret: 1, time: 1000, sustain: 100, bend_values: bv1, mask: NoteMask::LINK_NEXT, ..Default::default() },
        Note { fret: 1, time: 1100, sustain: 100, bend_values: bv2, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert!(results.is_empty());
}

#[test]
fn detects_hopo_into_same_note() {
    let notes = vec![
        Note { fret: 5, time: 1000, ..Default::default() },
        Note { fret: 5, time: 1200, mask: NoteMask::HAMMER_ON, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::HopoIntoSameNote));
}

#[test]
fn fret_numbers_over_24_are_detected() {
    let notes = vec![
        Note { fret: 24, time: 1000, ..Default::default() },
        Note { fret: 25, time: 2000, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::FretNumberMoreThan24));
    assert_eq!(results[0].time_code(), Some(2000));
}

#[test]
fn detects_note_after_end_phrase() {
    let notes = vec![Note { fret: 1, time: 50_000, ..Default::default() }];
    let level = Level { notes: notes.clone(), ..Default::default() };
    let phrases = vec![
        Phrase { name: "COUNT".into(), ..Default::default() },
        Phrase { name: "END".into(), ..Default::default() },
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
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NoteAfterSongEnd));
    assert_eq!(results[0].time_code(), Some(50_000));
}

#[test]
fn detects_notes_with_techniques_that_require_sustain() {
    let notes = vec![
        Note { fret: 1, time: 1_000, slide_to: 2, sustain: 0, ..Default::default() },
        Note { fret: 4, time: 4_000, slide_unpitch_to: 7, sustain: 0, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert!(results.iter().all(|r| matches!(r.issue_type(), IssueType::TechniqueNoteWithoutSustain)));
    assert!(!results.is_empty());
}

#[test]
fn overlapping_bend_values_are_detected() {
    let notes = vec![
        Note {
            fret: 3,
            sustain: 500,
            bend_values: vec![BendValue { time: 500, step: 1.0, ..Default::default() }, BendValue { time: 500, step: 1.0, ..Default::default() }],
            ..Default::default()
        },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::OverlappingBendValues));
}

#[test]
fn natural_harmonic_with_bend_is_detected() {
    let notes = vec![
        Note {
            fret: 12,
            sustain: 1000,
            max_bend: 2.0,
            bend_values: vec![BendValue { time: 500, step: 2.0, ..Default::default() }],
            mask: NoteMask::HARMONIC,
            ..Default::default()
        },
    ];
    let level = Level { notes, ..Default::default() };
    let arr = test_arr();
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].issue_type(), IssueType::NaturalHarmonicWithBend));
}

#[test]
fn invalid_strings_on_bass_arrangement_are_detected() {
    let notes = vec![
        Note { string: 4, time: 1000, ..Default::default() },
        Note { string: 5, time: 2000, ..Default::default() },
    ];
    let level = Level { notes: notes.clone(), ..Default::default() };
    let mut arr = InstrumentalArrangement {
        levels: vec![level.clone()],
        meta: MetaData { song_length: 500_000, ..Default::default() },
        ..Default::default()
    };
    arr.meta.arrangement_properties.path_bass = 1;
    let results = check_notes(&arr, &level);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| matches!(r.issue_type(), IssueType::InvalidBassArrangementString)));
}
