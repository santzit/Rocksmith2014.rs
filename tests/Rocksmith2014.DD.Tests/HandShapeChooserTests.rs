use rocksmith2014_dd::hand_shape_chooser;
use rocksmith2014_xml::{ChordTemplate, HandShape, Note};
use rocksmith2014_xml_extension::XmlEntity;

fn template() -> ChordTemplate {
    ChordTemplate {
        fingers: [1, 3, 4, -1, -1, -1],
        frets: [3, 5, 5, -1, -1, -1],
        ..Default::default()
    }
}

#[test]
fn chooses_none_when_no_notes_inside_handshape() {
    let templates = vec![template()];
    let hand_shapes = vec![HandShape {
        chord_id: 0,
        start_time: 2000,
        end_time: 3500,
    }];

    let result = hand_shape_chooser::choose(1.0, &[], &[], 3, &templates, &hand_shapes);
    assert_eq!(result.len(), 0);
}

#[test]
fn chooses_full_hand_shape_for_arpeggio() {
    let n1 = Note {
        string: 0,
        time: 2000,
        fret: 3,
        ..Default::default()
    };
    let n2 = Note {
        string: 1,
        time: 2500,
        fret: 5,
        ..Default::default()
    };
    let n3 = Note {
        string: 2,
        time: 3000,
        fret: 5,
        ..Default::default()
    };
    let entities = vec![
        XmlEntity::Note(n1),
        XmlEntity::Note(n2),
        XmlEntity::Note(n3),
    ];
    let templates = vec![template()];
    let hand_shapes = vec![HandShape {
        chord_id: 0,
        start_time: 2000,
        end_time: 3500,
    }];

    let result = hand_shape_chooser::choose(0.0, &entities, &entities, 3, &templates, &hand_shapes);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].1.is_none(), true);
    assert_eq!(result[0].0.chord_id, 0);
}

#[test]
fn chooses_hand_shape_when_a_note_sustain_ends_inside_it() {
    let templates = vec![template()];
    let hand_shapes = vec![HandShape {
        chord_id: 0,
        start_time: 2000,
        end_time: 3500,
    }];
    let level_entities = vec![XmlEntity::Note(Note {
        fret: 1,
        time: 0,
        sustain: 2000,
        ..Default::default()
    })];

    let result =
        hand_shape_chooser::choose(1.0, &level_entities, &level_entities, 3, &templates, &hand_shapes);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].1.is_none(), true);
}
