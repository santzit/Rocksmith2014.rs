use rocksmith2014_dd::{
    generator,
    types::{GeneratorConfig, LevelCountGeneration},
};
use rocksmith2014_xml::{
    Chord, ChordNote, ChordTemplate, InstrumentalArrangement, Level, MetaData, Note, Phrase,
    PhraseIteration,
};

fn config() -> GeneratorConfig {
    GeneratorConfig {
        phrase_search_threshold: Some(85),
        level_count_generation: LevelCountGeneration::Simple,
    }
}

fn arrangement(levels: Vec<Level>, chord_templates: Vec<ChordTemplate>) -> InstrumentalArrangement {
    InstrumentalArrangement {
        levels,
        chord_templates,
        phrases: vec![
            Phrase {
                name: "COUNT".to_string(),
                ..Default::default()
            },
            Phrase {
                name: "riff".to_string(),
                ..Default::default()
            },
            Phrase {
                name: "END".to_string(),
                ..Default::default()
            },
        ],
        phrase_iterations: vec![
            PhraseIteration {
                time: 0,
                phrase_id: 0,
                ..Default::default()
            },
            PhraseIteration {
                time: 5000,
                phrase_id: 1,
                ..Default::default()
            },
            PhraseIteration {
                time: 9000,
                phrase_id: 2,
                ..Default::default()
            },
        ],
        meta: MetaData {
            song_length: 10_000,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn creates_difficulty_levels() {
    let notes = vec![Note {
        time: 5000,
        ..Default::default()
    }];
    let level = Level {
        difficulty: 0,
        notes,
        ..Default::default()
    };

    let mut arr = arrangement(vec![level], vec![]);
    generator::generate_for_arrangement(config(), &mut arr);

    assert_eq!(arr.phrases[1].name, "p0");
    assert!(arr.levels.len() > 1);
}

#[test]
fn chord_is_replaced_with_note_in_low_level() {
    let cn = vec![
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
        chord_id: 0,
        chord_notes: cn,
        ..Default::default()
    }];
    let handshapes = vec![rocksmith2014_xml::HandShape {
        chord_id: 0,
        start_time: 5000,
        end_time: 800,
    }];
    let level = Level {
        difficulty: 0,
        chords,
        hand_shapes: handshapes,
        ..Default::default()
    };
    let template = ChordTemplate {
        name: "G5".to_string(),
        display_name: "G5".to_string(),
        fingers: [1, 3, 4, -1, -1, -1],
        frets: [3, 5, 5, -1, -1, -1],
    };

    let mut conf = config();
    conf.level_count_generation = LevelCountGeneration::Constant(10);
    let mut arr = arrangement(vec![level], vec![template]);
    generator::generate_for_arrangement(conf, &mut arr);

    let highest_level = &arr.levels[9];
    assert_eq!(arr.levels[0].notes.len(), 1);
    assert_eq!(arr.levels[0].notes[0].sustain, 0);
    assert_eq!(arr.levels[2].notes[0].sustain, 800);
    assert_eq!(highest_level.notes.len(), 0);
    assert_eq!(highest_level.chords.len(), 1);
}

#[test]
fn note_on_lowest_level_for_double_stop_on_high_strings_is_the_highest_string() {
    let cn = vec![
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
        chord_id: 0,
        chord_notes: cn,
        ..Default::default()
    }];
    let handshapes = vec![rocksmith2014_xml::HandShape {
        chord_id: 0,
        start_time: 5000,
        end_time: 800,
    }];
    let level = Level {
        difficulty: 0,
        chords,
        hand_shapes: handshapes,
        ..Default::default()
    };
    let template = ChordTemplate {
        fingers: [-1, -1, -1, -1, 1, 1],
        frets: [-1, -1, -1, -1, 5, 5],
        ..Default::default()
    };

    let mut conf = config();
    conf.level_count_generation = LevelCountGeneration::Constant(10);
    let mut arr = arrangement(vec![level], vec![template]);
    generator::generate_for_arrangement(conf, &mut arr);

    assert_eq!(arr.levels[0].notes.len(), 1);
    assert_eq!(arr.levels[0].notes[0].string, 5);
}
