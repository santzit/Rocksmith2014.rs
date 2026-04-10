use rocksmith2014_xml::{Chord, Ebeat, HandShape, InstrumentalArrangement, Level, Note};
use rocksmith2014_xml_processing::improvers::handshape_adjuster::{lengthen_handshapes, shorten_handshapes};

fn beats() -> Vec<Ebeat> {
    vec![
        Ebeat { time: 500, measure: -1 },
        Ebeat { time: 1000, measure: -1 },
        Ebeat { time: 1500, measure: -1 },
        Ebeat { time: 2500, measure: -1 },
    ]
}

#[test]
fn shortens_handshape_length() {
    let chords = vec![Chord { time: 1000, ..Default::default() }, Chord { chord_id: 1, time: 2000, ..Default::default() }];
    let hs1 = HandShape { chord_id: 0, start_time: 1000, end_time: 2000 };
    let hs2 = HandShape { chord_id: 1, start_time: 2000, end_time: 3000 };
    let mut arr = InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level { chords, hand_shapes: vec![hs1, hs2], ..Default::default() }],
        ..Default::default()
    };
    shorten_handshapes(&mut arr);
    assert!(arr.levels[0].hand_shapes[0].end_time < 2000);
    assert!(arr.levels[0].hand_shapes[0].start_time < arr.levels[0].hand_shapes[0].end_time);
}

#[test]
fn does_not_fail_on_handshapes_that_exceed_the_last_beat() {
    let hs1 = HandShape { chord_id: 0, start_time: 2500, end_time: 2600 };
    let hs2 = HandShape { chord_id: 0, start_time: 2600, end_time: 2800 };
    let mut arr = InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level { hand_shapes: vec![hs1, hs2], ..Default::default() }],
        ..Default::default()
    };
    shorten_handshapes(&mut arr);
    assert!(arr.levels[0].hand_shapes[0].end_time < 2600);
}

#[test]
fn lengthens_handshape_when_chord_is_at_end_of_handshape() {
    let chords = vec![
        Chord { time: 1000, ..Default::default() },
        Chord { time: 2000, ..Default::default() },
    ];
    let hs1 = HandShape { chord_id: 0, start_time: 1000, end_time: 2000 };
    let mut arr = InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level { chords, hand_shapes: vec![hs1], ..Default::default() }],
        ..Default::default()
    };
    lengthen_handshapes(&mut arr);
    assert_eq!(arr.levels[0].hand_shapes[0].end_time, 2250);
}

#[test]
fn lengthens_handshape_when_next_note_is_very_close() {
    let chords = vec![
        Chord { time: 1000, ..Default::default() },
        Chord { time: 2000, ..Default::default() },
    ];
    let notes = vec![Note { time: 2050, ..Default::default() }];
    let hs1 = HandShape { chord_id: 0, start_time: 1000, end_time: 2000 };
    let mut arr = InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level { chords, notes, hand_shapes: vec![hs1], ..Default::default() }],
        ..Default::default()
    };
    lengthen_handshapes(&mut arr);
    assert_eq!(arr.levels[0].hand_shapes[0].end_time, 2025);
}
