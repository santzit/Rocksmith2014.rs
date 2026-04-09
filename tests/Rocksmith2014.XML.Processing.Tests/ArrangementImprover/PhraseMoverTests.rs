use rocksmith2014_xml::{Chord, InstrumentalArrangement, Level, Note, Phrase, PhraseIteration, Section};
use rocksmith2014_xml_processing::improve_phrase_mover;

#[test]
fn can_move_phrase_to_next_note() {
    let notes = vec![Note { time: 1200, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        phrases: vec![Phrase { name: "mover1".into(), ..Default::default() }],
        phrase_iterations: vec![PhraseIteration { time: 1000, phrase_id: 0, ..Default::default() }],
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    improve_phrase_mover(&mut arr);
    assert_eq!(arr.phrase_iterations[0].time, 1200);
}

#[test]
fn can_move_phrase_to_chord() {
    let notes = vec![Note { time: 1200, ..Default::default() }];
    let chords = vec![Chord { time: 1600, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        phrases: vec![Phrase { name: "mover2".into(), ..Default::default() }],
        phrase_iterations: vec![PhraseIteration { time: 1000, phrase_id: 0, ..Default::default() }],
        levels: vec![Level { notes, chords, ..Default::default() }],
        ..Default::default()
    };
    improve_phrase_mover(&mut arr);
    assert_eq!(arr.phrase_iterations[0].time, 1600);
}

#[test]
fn section_is_also_moved() {
    let notes = vec![Note { time: 7500, ..Default::default() }];
    let section = Section { name: "".into(), start_time: 1000, number: 1, end_time: 0 };
    let mut arr = InstrumentalArrangement {
        phrases: vec![Phrase { name: "mover1".into(), ..Default::default() }],
        phrase_iterations: vec![PhraseIteration { time: 1000, phrase_id: 0, ..Default::default() }],
        sections: vec![section],
        levels: vec![Level { notes, ..Default::default() }],
        ..Default::default()
    };
    improve_phrase_mover(&mut arr);
    assert_eq!(arr.phrase_iterations[0].time, 7500);
    assert_eq!(arr.sections[0].start_time, 7500);
}
