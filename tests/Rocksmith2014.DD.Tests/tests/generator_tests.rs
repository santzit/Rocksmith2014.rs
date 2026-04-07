use rocksmith2014_dd::{generate_for_arrangement, GeneratorConfig, LevelCountGeneration};
use rocksmith2014_xml::{
    Chord, ChordNote, ChordTemplate, HandShape, InstrumentalArrangement, Level, MetaData, Note,
    Phrase, PhraseIteration,
};

fn make_arr(levels: Vec<Level>, chord_templates: Vec<ChordTemplate>) -> InstrumentalArrangement {
    InstrumentalArrangement {
        phrases: vec![
            Phrase {
                name: "COUNT".into(),
                ..Default::default()
            },
            Phrase {
                name: "riff".into(),
                ..Default::default()
            },
            Phrase {
                name: "END".into(),
                ..Default::default()
            },
        ],
        phrase_iterations: vec![
            PhraseIteration {
                time: 0,
                end_time: 5000,
                phrase_id: 0,
                ..Default::default()
            },
            PhraseIteration {
                time: 5000,
                end_time: 9000,
                phrase_id: 1,
                ..Default::default()
            },
            PhraseIteration {
                time: 9000,
                end_time: 10000,
                phrase_id: 2,
                ..Default::default()
            },
        ],
        meta: MetaData {
            song_length: 10_000,
            ..Default::default()
        },
        levels,
        chord_templates,
        ..Default::default()
    }
}

#[test]
fn creates_difficulty_levels() {
    let config = GeneratorConfig {
        phrase_search_threshold: Some(85),
        level_count_generation: LevelCountGeneration::Simple,
    };
    let notes = vec![Note {
        time: 5000,
        ..Default::default()
    }];
    let level = Level {
        notes,
        ..Default::default()
    };
    let mut arr = make_arr(vec![level], vec![]);
    generate_for_arrangement(&config, &mut arr);
    assert!(arr.levels.len() > 1, "DD levels should be generated");
}

#[test]
fn chord_is_replaced_with_note_in_low_level() {
    let config = GeneratorConfig {
        phrase_search_threshold: Some(85),
        level_count_generation: LevelCountGeneration::Constant(10),
    };
    let chord_notes = vec![
        ChordNote {
            string: 0,
            fret: 3,
            sustain: 800,
            ..Default::default()
        },
        ChordNote {
            string: 1,
            fret: 5,
            sustain: 800,
            ..Default::default()
        },
        ChordNote {
            string: 2,
            fret: 5,
            sustain: 800,
            ..Default::default()
        },
    ];
    let chords = vec![Chord {
        time: 5000,
        chord_notes,
        ..Default::default()
    }];
    let hand_shapes = vec![HandShape {
        chord_id: 0,
        start_time: 5000,
        end_time: 5800,
    }];
    let level = Level {
        chords,
        hand_shapes,
        ..Default::default()
    };
    let template = ChordTemplate {
        chord_name: "G5".into(),
        display_name: "G5".into(),
        fingers: [1, 3, 4, -1, -1, -1],
        frets: [3, 5, 5, -1, -1, -1],
    };
    let mut arr = make_arr(vec![level], vec![template]);
    generate_for_arrangement(&config, &mut arr);
    assert_eq!(arr.levels.len(), 10, "10 levels should be generated");
    // Lowest level should have a note (chord converted to note)
    assert!(
        !arr.levels[0].notes.is_empty(),
        "note should exist in lowest level"
    );
    // Highest level should have chord
    assert!(
        !arr.levels[9].chords.is_empty(),
        "chord should exist in highest level"
    );
}

#[test]
fn double_stop_note_on_low_level_is_highest_string() {
    let config = GeneratorConfig {
        phrase_search_threshold: Some(85),
        level_count_generation: LevelCountGeneration::Constant(10),
    };
    let chord_notes = vec![
        ChordNote {
            string: 4,
            fret: 5,
            sustain: 800,
            ..Default::default()
        },
        ChordNote {
            string: 5,
            fret: 5,
            sustain: 800,
            ..Default::default()
        },
    ];
    let chords = vec![Chord {
        time: 5000,
        chord_notes,
        ..Default::default()
    }];
    let hand_shapes = vec![HandShape {
        chord_id: 0,
        start_time: 5000,
        end_time: 5800,
    }];
    let level = Level {
        chords,
        hand_shapes,
        ..Default::default()
    };
    let template = ChordTemplate {
        chord_name: "".into(),
        display_name: "".into(),
        fingers: [-1, -1, -1, -1, 1, 1],
        frets: [-1, -1, -1, -1, 5, 5],
    };
    let mut arr = make_arr(vec![level], vec![template]);
    generate_for_arrangement(&config, &mut arr);
    assert!(
        !arr.levels[0].notes.is_empty(),
        "note should exist in lowest level"
    );
    assert_eq!(
        arr.levels[0].notes[0].string, 5,
        "note should be on string 5 (highest)"
    );
}
