use rocksmith2014_xml::{
    Anchor, Chord, ChordNote, ChordTemplate, Ebeat, HandShape, InstrumentalArrangement, Level, Note,
};
use rocksmith2014_xml_processing::improvers::phrase_gen::generate_phrases;

fn beats() -> Vec<Ebeat> {
    (0..15)
        .map(|i| Ebeat {
            time: (i + 1) * 1000,
            measure: i as i16,
        })
        .collect()
}

fn base_arr() -> InstrumentalArrangement {
    InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level::default()],
        ..Default::default()
    }
}

#[test]
fn nothing_is_generated_for_arrangement_without_any_levels() {
    let mut arr = InstrumentalArrangement::default();
    generate_phrases(&mut arr);
    assert!(arr.phrases.is_empty());
    assert!(arr.sections.is_empty());
}

#[test]
fn creates_phrases_and_sections_when_one_note() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    generate_phrases(&mut arr);
    assert!(!arr.phrases.is_empty());
    assert!(!arr.phrase_iterations.is_empty());
    assert!(!arr.sections.is_empty());
    assert_eq!(arr.phrase_iterations[0].time, 1000);
    assert_eq!(arr.phrase_iterations[1].time, 2000);
    assert_eq!(arr.sections[0].start_time, 2000);
}

#[test]
fn does_not_create_phrase_in_middle_of_note_sustain() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    arr.levels[0].notes.push(Note {
        time: 9_500,
        sustain: 3_000,
        ..Default::default()
    });
    generate_phrases(&mut arr);
    assert!(arr.phrase_iterations.iter().any(|pi| pi.time == 9500));
    assert!(arr.sections.iter().any(|s| s.start_time == 9500));
}

#[test]
fn does_not_create_end_phrase_on_the_last_note() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 6_000,
        ..Default::default()
    });
    generate_phrases(&mut arr);
    let end_phrase = arr.phrase_iterations.last().unwrap();
    assert_ne!(end_phrase.time, 6_000);
}

#[test]
fn does_not_create_a_phrase_in_the_middle_of_a_handshape() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    arr.levels[0].chords.push(Chord {
        time: 8_500,
        ..Default::default()
    });
    arr.levels[0].hand_shapes.push(HandShape {
        chord_id: 0,
        start_time: 8_500,
        end_time: 10_800,
    });
    generate_phrases(&mut arr);
    // The end time is closer to the time where the phrase would be created (10s)
    // Phrase should be at the handshape end time (10800), not inside the handshape
    assert!(arr.phrase_iterations.iter().any(|pi| pi.time == 10_800));
    assert!(arr.sections.iter().any(|s| s.start_time == 10_800));
}

#[test]
fn does_not_create_a_phrase_that_breaks_note_link_next() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    arr.levels[0].notes.push(Note {
        time: 9_500,
        sustain: 500,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 10_000,
        sustain: 1000,
        ..Default::default()
    });
    generate_phrases(&mut arr);
    // Possible good phrase times are 9.5s and 11s; 9.5s is closer to 10s
    assert_eq!(arr.phrase_iterations[2].time, 9500);
    assert_eq!(arr.sections[1].start_time, 9500);
}

#[test]
fn does_not_create_a_phrase_that_breaks_note_link_next_no_sustain_on_linknext_target_note() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    arr.levels[0].notes.push(Note {
        time: 9_500,
        sustain: 500,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 10_000,
        sustain: 0,
        ..Default::default()
    });
    generate_phrases(&mut arr);
    // When the linknext target note has no sustain, candidate time is note time + 100ms
    assert_eq!(arr.phrase_iterations[2].time, 10_100);
    assert_eq!(arr.sections[1].start_time, 10_100);
}

#[test]
fn does_not_create_a_phrase_that_breaks_note_link_next_multiple_link_next_notes() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    arr.levels[0].notes.push(Note {
        time: 9_200,
        sustain: 300,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 9_500,
        sustain: 500,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 10_000,
        sustain: 500,
        ..Default::default()
    });
    generate_phrases(&mut arr);
    // Possible good phrase times are 9.2s and 10.5s; 10.5s is closer to 10s
    assert_eq!(arr.phrase_iterations[2].time, 10_500);
    assert_eq!(arr.sections[1].start_time, 10_500);
}

#[test]
fn does_not_create_a_phrase_that_breaks_chord_link_next() {
    let mut arr = base_arr();
    arr.chord_templates.push(ChordTemplate {
        name: "".into(),
        display_name: "".into(),
        fingers: [1, -1, -1, -1, -1, -1],
        frets: [1, -1, -1, -1, -1, -1],
    });
    arr.levels[0].notes.push(Note {
        time: 2_000,
        ..Default::default()
    });
    arr.levels[0].anchors.push(Anchor {
        fret: 1,
        time: 2_000,
        width: 4,
        end_time: 0,
    });
    let cn = vec![ChordNote {
        sustain: 500,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    }];
    arr.levels[0].chords.push(Chord {
        time: 9_500,
        chord_notes: cn,
        mask: rocksmith2014_xml::ChordMask::LINK_NEXT,
        ..Default::default()
    });
    arr.levels[0].notes.push(Note {
        time: 10_000,
        sustain: 700,
        ..Default::default()
    });
    // Note: handshape is shorter than chord note sustain
    arr.levels[0].hand_shapes.push(HandShape {
        chord_id: 0,
        start_time: 9_500,
        end_time: 9_800,
    });
    generate_phrases(&mut arr);
    // Possible good phrase times are 9.5s and 10.7s; 9.5s is closer to 10s
    assert_eq!(arr.phrase_iterations[2].time, 9500);
    assert_eq!(arr.sections[1].start_time, 9500);
}
