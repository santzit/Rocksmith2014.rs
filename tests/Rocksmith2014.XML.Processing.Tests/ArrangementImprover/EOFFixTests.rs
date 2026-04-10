use rocksmith2014_xml::{
    ArrangementEvent, Chord, ChordMask, ChordNote, InstrumentalArrangement, Level, Note, NoteMask,
};
use rocksmith2014_xml_processing::improvers::eof_fixes::{
    fix_chord_notes as eof_fix_chord_notes,
    fix_chord_slide_handshapes as eof_fix_chord_slide_handshapes,
    fix_crowd_events as eof_fix_crowd_events,
    remove_invalid_chord_note_link_nexts as eof_remove_invalid_chord_note_link_nexts,
};

#[test]
fn adds_linknext_to_chords_missing_the_attribute() {
    let chord = Chord {
        chord_notes: vec![ChordNote { mask: NoteMask::LINK_NEXT, ..Default::default() }],
        ..Default::default()
    };
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords: vec![chord], ..Default::default() }],
        ..Default::default()
    };
    eof_fix_chord_notes(&mut arr);
    assert!(arr.levels[0].chords[0].mask.contains(ChordMask::LINK_NEXT));
}

#[test]
fn fixes_varying_sustain_of_chord_notes() {
    let chord = Chord {
        chord_notes: vec![
            ChordNote { sustain: 0, ..Default::default() },
            ChordNote { string: 1, sustain: 500, ..Default::default() },
            ChordNote { string: 2, sustain: 85, ..Default::default() },
        ],
        ..Default::default()
    };
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords: vec![chord], ..Default::default() }],
        ..Default::default()
    };
    eof_fix_chord_notes(&mut arr);
    assert!(arr.levels[0].chords[0].chord_notes.iter().all(|cn| cn.sustain == 500));
}

#[test]
fn removes_incorrect_chord_note_linknexts() {
    let cn = vec![ChordNote { mask: NoteMask::LINK_NEXT, ..Default::default() }];
    let chords = vec![Chord { chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords, ..Default::default() }],
        ..Default::default()
    };
    eof_remove_invalid_chord_note_link_nexts(&mut arr);
    assert!(!arr.levels[0].chords[0].chord_notes[0].mask.contains(NoteMask::LINK_NEXT));
}

#[test]
fn chord_note_linknext_is_not_removed_when_there_is_1ms_gap() {
    let cn = vec![
        ChordNote { string: 0, sustain: 499, mask: NoteMask::LINK_NEXT, ..Default::default() },
        ChordNote { string: 1, sustain: 499, mask: NoteMask::LINK_NEXT, ..Default::default() },
    ];
    let chords = vec![Chord { chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() }];
    let notes = vec![Note { string: 0, time: 500, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords, notes, ..Default::default() }],
        ..Default::default()
    };
    eof_remove_invalid_chord_note_link_nexts(&mut arr);
    assert!(arr.levels[0].chords[0].chord_notes[0].mask.contains(NoteMask::LINK_NEXT));
    assert!(!arr.levels[0].chords[0].chord_notes[1].mask.contains(NoteMask::LINK_NEXT));
}

#[test]
fn fixes_incorrect_crowd_events() {
    let events = vec![
        ArrangementEvent { code: "E0".into(), time: 100 },
        ArrangementEvent { code: "E1".into(), time: 200 },
        ArrangementEvent { code: "E2".into(), time: 300 },
    ];
    let mut arr = InstrumentalArrangement { events, ..Default::default() };
    eof_fix_crowd_events(&mut arr);
    assert_eq!(arr.events.len(), 3);
    assert!(arr.events.iter().any(|e| e.code == "e0"));
    assert!(arr.events.iter().any(|e| e.code == "e1"));
    assert!(arr.events.iter().any(|e| e.code == "e2"));
}

#[test]
fn does_not_change_correct_crowd_events() {
    let events = vec![
        ArrangementEvent { code: "E3".into(), time: 100 },
        ArrangementEvent { code: "E13".into(), time: 200 },
        ArrangementEvent { code: "D3".into(), time: 300 },
        ArrangementEvent { code: "E13".into(), time: 400 },
    ];
    let mut arr = InstrumentalArrangement { events, ..Default::default() };
    eof_fix_crowd_events(&mut arr);
    assert_eq!(arr.events.len(), 4);
    assert_eq!(arr.events[0].code, "E3");
    assert_eq!(arr.events[1].code, "E13");
    assert_eq!(arr.events[2].code, "D3");
    assert_eq!(arr.events[3].code, "E13");
}

#[test]
fn fixes_incorrect_handshape_lengths() {
    use rocksmith2014_xml::{HandShape, ChordMask};
    let cn = vec![ChordNote { slide_to: 5, sustain: 1000, mask: NoteMask::LINK_NEXT, ..Default::default() }];
    let chord = Chord { chord_notes: cn, mask: ChordMask::LINK_NEXT, ..Default::default() };
    let hs = HandShape { chord_id: 0, start_time: 0, end_time: 1500 };
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { chords: vec![chord], hand_shapes: vec![hs], ..Default::default() }],
        ..Default::default()
    };
    eof_fix_chord_slide_handshapes(&mut arr);
    assert_eq!(arr.levels[0].hand_shapes[0].end_time, 1000);
}

#[test]
fn moves_anchor_to_the_beginning_of_phrase() {
    use rocksmith2014_xml::{Anchor, PhraseIteration};
    let anchor = Anchor { fret: 5, time: 700, width: 4, end_time: 0 };
    let phrase_iterations = vec![
        PhraseIteration { time: 100, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 650, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 1000, phrase_id: 1, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { anchors: vec![anchor], ..Default::default() }],
        phrase_iterations,
        ..Default::default()
    };
    use rocksmith2014_xml_processing::improvers::eof_fixes::fix_phrase_start_anchors;
    fix_phrase_start_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 1);
    assert_eq!(arr.levels[0].anchors[0].time, 650);
}

#[test]
fn copies_active_anchor_to_the_beginning_of_phrase() {
    use rocksmith2014_xml::{Anchor, PhraseIteration};
    let anchor = Anchor { fret: 5, time: 400, width: 7, end_time: 0 };
    let phrase_iterations = vec![
        PhraseIteration { time: 100, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 400, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 650, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 1000, phrase_id: 1, ..Default::default() },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level { anchors: vec![anchor], ..Default::default() }],
        phrase_iterations,
        ..Default::default()
    };
    use rocksmith2014_xml_processing::improvers::eof_fixes::fix_phrase_start_anchors;
    fix_phrase_start_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 2);
    assert_eq!(arr.levels[0].anchors[0].time, 400);
    assert_eq!(arr.levels[0].anchors[1].time, 650);
    assert_eq!(arr.levels[0].anchors[1].fret, 5);
    assert_eq!(arr.levels[0].anchors[1].width, 7);
}
