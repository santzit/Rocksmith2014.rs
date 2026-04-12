use rocksmith2014_dd::anchor_chooser;
use rocksmith2014_xml::{Anchor, Note};
use rocksmith2014_xml_extension::XmlEntity;

#[test]
fn chooses_anchor_at_the_start_of_the_phrase() {
    let anchors = vec![Anchor {
        fret: 0,
        time: 1050,
        ..Default::default()
    }];
    let entities = vec![XmlEntity::Note(Note {
        time: 1200,
        ..Default::default()
    })];

    let result = anchor_chooser::choose(&entities, &anchors, 1050, 2000);
    assert_eq!(result.len(), 1);
}

#[test]
fn chooses_anchor_at_the_end_of_sustain() {
    let anchors = vec![Anchor {
        fret: 0,
        time: 1350,
        ..Default::default()
    }];
    let entities = vec![XmlEntity::Note(Note {
        time: 1200,
        fret: 1,
        sustain: 150,
        slide_to: 2,
        ..Default::default()
    })];

    let result = anchor_chooser::choose(&entities, &anchors, 1050, 2000);
    assert_eq!(result.len(), 1);
}
