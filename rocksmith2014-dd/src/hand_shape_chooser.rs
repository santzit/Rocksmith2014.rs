use crate::types::TemplateRequest;
use crate::utils::{create_template_request, get_allowed_chord_notes, get_note_count};
use rocksmith2014_xml::{ChordTemplate, HandShape};
use rocksmith2014_xml_extension::XmlEntity;

fn is_inside_hand_shape(hs: &HandShape, time: i32) -> bool {
    time >= hs.start_time && time < hs.end_time
}

fn no_notes_in_hand_shape(entities: &[XmlEntity], hs: &HandShape) -> bool {
    !entities.iter().any(|x| {
        let time = x.time_code();
        is_inside_hand_shape(hs, time) || is_inside_hand_shape(hs, time + x.sustain())
    })
}

fn is_arpeggio(entities: &[XmlEntity], hs: &HandShape) -> bool {
    let hand_shape_notes = entities
        .iter()
        .filter_map(|e| match e {
            XmlEntity::Note(n) if is_inside_hand_shape(hs, n.time) => Some(n),
            _ => None,
        })
        .collect::<Vec<_>>();

    if hand_shape_notes.is_empty() {
        return false;
    }

    !hand_shape_notes
        .iter()
        .all(|n| n.string == hand_shape_notes[0].string || n.time == hand_shape_notes[0].time)
}

pub fn choose(
    diff_percent: f64,
    level_entities: &[XmlEntity],
    all_entities: &[XmlEntity],
    max_chord_notes: usize,
    templates: &[ChordTemplate],
    hand_shapes: &[HandShape],
) -> Vec<(HandShape, Option<TemplateRequest>)> {
    let allowed_notes = get_allowed_chord_notes(diff_percent, max_chord_notes);

    hand_shapes
        .iter()
        .filter_map(|hs| {
            let template = templates.get(hs.chord_id as usize)?;
            let note_count = get_note_count(template);

            if no_notes_in_hand_shape(level_entities, hs) {
                None
            } else if is_arpeggio(all_entities, hs) {
                Some((hs.clone(), None))
            } else if allowed_notes <= 1 {
                None
            } else if allowed_notes >= note_count {
                Some((hs.clone(), None))
            } else {
                let copy = hs.clone();
                let request = create_template_request(
                    hs.chord_id as i16,
                    allowed_notes,
                    note_count,
                    template,
                    crate::types::RequestTarget::HandShapeTarget(copy.clone()),
                );
                Some((copy, Some(request)))
            }
        })
        .collect()
}
