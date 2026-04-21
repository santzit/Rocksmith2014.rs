use crate::types::{RequestTarget, TemplateRequest};
use rocksmith2014_xml::ChordTemplate;

pub fn get_note_count(template: &ChordTemplate) -> usize {
    template.frets.iter().filter(|&&f| f >= 0).count()
}

pub fn get_allowed_chord_notes(diff_percent: f64, max_chord_notes_in_phrase: usize) -> usize {
    if max_chord_notes_in_phrase == 0 {
        0
    } else {
        (diff_percent * max_chord_notes_in_phrase as f64).ceil() as usize
    }
}

pub fn should_start_from_highest_note(total_notes: usize, template: &ChordTemplate) -> bool {
    let fr = template.frets;
    (total_notes == 3 && fr[5] > -1 && fr[4] > -1 && fr[3] > -1)
        || (total_notes == 2 && fr[5] > -1 && fr[4] > -1)
}

pub fn create_template_request(
    original_id: i16,
    note_count: usize,
    total_notes: usize,
    template: &ChordTemplate,
    target: RequestTarget,
) -> TemplateRequest {
    TemplateRequest {
        original_id,
        note_count: note_count as u8,
        from_highest_note: should_start_from_highest_note(total_notes, template),
        target,
    }
}
