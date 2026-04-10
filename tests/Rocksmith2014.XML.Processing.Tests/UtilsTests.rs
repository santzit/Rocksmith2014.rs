use rocksmith2014_xml::{Chord, InstrumentalArrangement, Level, Note, Phrase, PhraseIteration};
use rocksmith2014_xml_processing::improvers::improver::get_first_note_time;

#[test]
fn get_first_note_time_does_not_fail_when_there_are_no_phrase_iterations() {
    let mut arr = InstrumentalArrangement::default();
    let notes = vec![Note { time: 4000, ..Default::default() }];
    arr.levels = vec![Level::default(), Level { notes, ..Default::default() }];
    let time = get_first_note_time(&arr);
    assert_eq!(time, Some(4000));
}

#[test]
fn get_first_note_time_finds_first_note_when_there_are_dd_levels() {
    let phrases = vec![
        Phrase { name: "default".into(), max_difficulty: 0, ..Default::default() },
        Phrase { name: "riff".into(), max_difficulty: 1, ..Default::default() },
    ];
    let phrase_iterations = vec![
        PhraseIteration { time: 0, phrase_id: 0, ..Default::default() },
        PhraseIteration { time: 5000, phrase_id: 1, ..Default::default() },
    ];
    let notes = vec![Note { time: 5400, ..Default::default() }];
    let chords = vec![Chord { time: 5000, ..Default::default() }];
    let mut arr = InstrumentalArrangement {
        phrases,
        phrase_iterations,
        ..Default::default()
    };
    arr.levels = vec![
        Level::default(),
        Level { notes, chords, ..Default::default() },
    ];
    let time = get_first_note_time(&arr);
    assert_eq!(time, Some(5000));
}
