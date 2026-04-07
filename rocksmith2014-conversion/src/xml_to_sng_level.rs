use rocksmith2014_sng::{FingerPrint, Level as SngLevel};
use rocksmith2014_xml::{InstrumentalArrangement, Level as XmlLevel};

use crate::{
    accu_data::AccuData,
    xml_to_sng::{convert_anchor, convert_handshape, create_entity_array, XmlEntity},
    xml_to_sng_note::{flag_on_anchor_change, NoteConverter},
};

/// Converts an XML Level to an SNG Level.
pub fn convert_level(
    accu: &mut AccuData,
    pi_times: &[i32],
    arr: &InstrumentalArrangement,
    level: &XmlLevel,
) -> SngLevel {
    let difficulty = level.difficulty as usize;

    let entities = create_entity_array(level);
    let note_times: Vec<i32> = entities.iter().map(|e| e.time()).collect();

    // Collect all notes as XmlNote for anchor lookups
    let xml_notes: Vec<rocksmith2014_xml::Note> = level.notes.clone();

    // Convert anchors
    let anchors: Vec<_> = level
        .anchors
        .iter()
        .enumerate()
        .map(|(i, a)| convert_anchor(&xml_notes, &note_times, level, arr, i, a))
        .collect();

    // Convert hand shapes and arpeggios
    let (hand_shapes, arpeggios) = build_finger_prints(&entities, &note_times, level, arr);

    // Convert notes with stateful converter
    let mut converter = NoteConverter::new(
        &note_times,
        pi_times,
        &hand_shapes,
        &arpeggios,
        accu,
        flag_on_anchor_change,
        arr,
        difficulty,
    );

    let sng_notes: Vec<_> = entities
        .into_iter()
        .enumerate()
        .map(|(i, e)| converter.call(i, e))
        .collect();

    // Compute average notes per phrase iteration and per phrase
    let phrase_count = arr.phrases.len();
    let pi_count = arr.phrase_iterations.len();

    let notes_per_pi = if difficulty < accu.notes_in_phrase_iterations_all.len() {
        accu.notes_in_phrase_iterations_all[difficulty].clone()
    } else {
        vec![0; pi_count]
    };

    let average_notes_per_iteration =
        compute_average_notes_per_phrase(arr, &notes_per_pi, phrase_count);

    let notes_excl = if difficulty < accu.notes_in_phrase_iterations_excl_ignored.len() {
        accu.notes_in_phrase_iterations_excl_ignored[difficulty].clone()
    } else {
        vec![0; pi_count]
    };

    let notes_all = notes_per_pi;

    SngLevel {
        difficulty: difficulty as i32,
        anchors,
        anchor_extensions: if difficulty < accu.anchor_extensions.len() {
            accu.anchor_extensions[difficulty].clone()
        } else {
            vec![]
        },
        hand_shapes,
        arpeggios,
        notes: sng_notes,
        average_notes_per_iteration,
        notes_in_phrase_iterations_excl_ignored: notes_excl,
        notes_in_phrase_iterations_all: notes_all,
    }
}

fn build_finger_prints(
    entities: &[XmlEntity],
    note_times: &[i32],
    level: &XmlLevel,
    arr: &InstrumentalArrangement,
) -> (Vec<FingerPrint>, Vec<FingerPrint>) {
    let mut hand_shapes = Vec::new();
    let mut arpeggios = Vec::new();

    for hs in &level.hand_shapes {
        // Determine if arpeggio by chord template
        let is_arp = arr
            .chord_templates
            .get(hs.chord_id as usize)
            .map(|t| t.display_name.ends_with("-arp"))
            .unwrap_or(false);

        let fp = convert_handshape(note_times, entities, hs);
        if is_arp {
            arpeggios.push(fp);
        } else {
            hand_shapes.push(fp);
        }
    }

    (hand_shapes, arpeggios)
}

fn compute_average_notes_per_phrase(
    arr: &InstrumentalArrangement,
    notes_per_pi: &[i32],
    phrase_count: usize,
) -> Vec<f32> {
    (0..phrase_count)
        .map(|phrase_id| {
            let notes_and_counts: Vec<f32> = arr
                .phrase_iterations
                .iter()
                .enumerate()
                .filter(|(_, pi)| pi.phrase_id == phrase_id as u32)
                .filter_map(|(pi_idx, _)| notes_per_pi.get(pi_idx).map(|&n| n as f32))
                .collect();

            if notes_and_counts.is_empty() {
                0.0
            } else {
                notes_and_counts.iter().sum::<f32>() / notes_and_counts.len() as f32
            }
        })
        .collect()
}
