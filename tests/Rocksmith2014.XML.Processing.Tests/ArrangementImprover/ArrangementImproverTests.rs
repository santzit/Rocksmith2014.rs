use rocksmith2014_xml::{Anchor, ArrangementEvent, ChordNote, Chord, Ebeat, InstrumentalArrangement, Level, Note, ChordTemplate};
use rocksmith2014_xml_processing::improver::{add_crowd_events, process_chord_names, remove_extra_beats};
use rocksmith2014_xml_processing::custom_events::improve as improve_custom_events;

#[test]
fn creates_crowd_events() {
    let notes = vec![Note { time: 10000, ..Default::default() }];
    let level = Level { notes, ..Default::default() };
    let mut arr = InstrumentalArrangement {
        levels: vec![level],
        ..Default::default()
    };
    arr.meta.song_length = 120_000;
    add_crowd_events(&mut arr);
    assert!(!arr.events.is_empty());
}

#[test]
fn no_events_are_created_when_already_present() {
    let notes = vec![Note { time: 10000, ..Default::default() }];
    let level = Level { notes, ..Default::default() };
    let events = vec![
        ArrangementEvent { code: "e1".into(), time: 1000 },
        ArrangementEvent { code: "E3".into(), time: 10000 },
        ArrangementEvent { code: "D3".into(), time: 20000 },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![level],
        events,
        ..Default::default()
    };
    arr.meta.song_length = 120_000;
    add_crowd_events(&mut arr);
    assert_eq!(arr.events.len(), 3);
}

#[test]
fn removes_beats() {
    let beats = vec![
        Ebeat { time: 5000, measure: 1 },
        Ebeat { time: 6000, measure: 1 },
        Ebeat { time: 7000, measure: 1 },
        Ebeat { time: 8000, measure: 1 },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6000;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 2);
}

#[test]
fn moves_beat_after_end_close_to_it_to_end() {
    let beats = vec![
        Ebeat { time: 5000, measure: 1 },
        Ebeat { time: 6000, measure: 1 },
        Ebeat { time: 7000, measure: 1 },
        Ebeat { time: 8000, measure: 1 },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6900;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 3);
    assert_eq!(arr.ebeats[2].time, 6900);
}

#[test]
fn moves_beat_before_end_close_to_it_to_end() {
    let beats = vec![
        Ebeat { time: 5000, measure: 1 },
        Ebeat { time: 6000, measure: 1 },
        Ebeat { time: 7000, measure: 1 },
        Ebeat { time: 8000, measure: 1 },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6100;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 2);
    assert_eq!(arr.ebeats[1].time, 6100);
}

#[test]
fn anchor_width_3_event() {
    let anchor = Anchor { time: 100, fret: 1, width: 4, end_time: 0 };
    let mut arr = InstrumentalArrangement {
        events: vec![ArrangementEvent { code: "w3".into(), time: 100 }],
        levels: vec![Level { anchors: vec![anchor], ..Default::default() }],
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].width, 3);
}

#[test]
fn remove_beats_event() {
    let beats: Vec<_> = (0..5).map(|i| Ebeat { time: 100 * (i + 1), measure: -1 }).collect();
    let mut arr = InstrumentalArrangement {
        events: vec![ArrangementEvent { code: "removebeats".into(), time: 400 }],
        ebeats: beats,
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    assert!(arr.ebeats.iter().all(|b| b.time < 400));
}

#[test]
fn fixes_minor_chord_names() {
    let c1 = ChordTemplate { name: "Emin".into(), display_name: "Emin".into(), fingers: [-1; 6], frets: [-1; 6] };
    let c2 = ChordTemplate { name: "Amin7".into(), display_name: "Amin7".into(), fingers: [-1; 6], frets: [-1; 6] };
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c1, c2],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert!(!arr.chord_templates[0].name.contains("min"));
    assert!(!arr.chord_templates[1].name.contains("min"));
}
