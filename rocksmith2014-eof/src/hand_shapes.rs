use rocksmith2014_xml::InstrumentalArrangement;
use crate::types::{EofNote, EofNoteFlag, EofSection, HsResult, SustainAdjustment};

fn hand_shape_not_needed(is_arpeggio: bool, notes_in_hs: &[&EofNote]) -> bool {
    match notes_in_hs.first() {
        None => false,
        Some(first) => {
            !is_arpeggio
                && notes_in_hs.iter().all(|n| n.bit_flag == first.bit_flag && !n.flags.contains(EofNoteFlag::SPLIT))
        }
    }
}

pub fn convert_hand_shapes(inst: &InstrumentalArrangement, notes: &[EofNote]) -> Vec<HsResult> {
    let mut results = Vec::new();

    for (diff, level) in inst.levels.iter().enumerate() {
        let diff = diff as u8;

        for hs in &level.hand_shapes {
            let notes_in_hs: Vec<&EofNote> = notes.iter()
                .filter(|n| {
                    n.difficulty == diff
                        && n.position as i32 >= hs.start_time
                        && (n.position as i32) < hs.end_time
                })
                .collect();

            let is_arpeggio = inst.chord_templates[hs.chord_id as usize].display_name.ends_with("-arp");

            if hand_shape_not_needed(is_arpeggio, &notes_in_hs) {
                let updates: Vec<SustainAdjustment> = notes_in_hs.iter().enumerate()
                    .map(|(i, n)| {
                        let new_sustain = match notes_in_hs.get(i + 1) {
                            Some(next) => next.position - n.position - 5,
                            None => hs.end_time as u32 - n.position,
                        };
                        SustainAdjustment {
                            difficulty: diff,
                            time: n.position,
                            new_sustain,
                        }
                    })
                    .collect();
                results.push(HsResult::AdjustSustains(updates));
            } else {
                let section = EofSection::create(
                    diff,
                    hs.start_time as u32,
                    hs.end_time as u32,
                    if is_arpeggio { 0 } else { 2 },
                );
                results.push(HsResult::SectionCreated(section));
            }
        }
    }

    results
}
