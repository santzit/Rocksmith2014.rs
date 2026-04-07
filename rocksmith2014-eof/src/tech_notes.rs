use crate::types::{EofExtendedNoteFlag, EofNote, EofNoteFlag};

fn can_move(tn: &EofNote) -> bool {
    !tn.flags.contains(EofNoteFlag::BEND)
        && !tn.extended_note_flags.contains(EofExtendedNoteFlag::STOP)
}

fn combine(current: EofNote, prev: &EofNote) -> EofNote {
    let mut frets = prev.frets.clone();
    frets.extend_from_slice(&current.frets);
    EofNote {
        bit_flag: current.bit_flag | prev.bit_flag,
        frets,
        extended_note_flags: current.extended_note_flags | prev.extended_note_flags,
        ..current
    }
}

fn move_position(tn: EofNote) -> EofNote {
    EofNote {
        position: (tn.position + 50).min(tn.end_position),
        ..tn
    }
}

fn convert_to_pre_bend(tn: EofNote) -> EofNote {
    EofNote {
        flags: tn.flags | EofNoteFlag::EXTENDED_FLAGS,
        extended_note_flags: tn.extended_note_flags | EofExtendedNoteFlag::PRE_BEND,
        ..tn
    }
}

fn is_combinable(a: &EofNote, b: &EofNote) -> bool {
    a.position == b.position
        && a.bit_flag != b.bit_flag
        && a.difficulty == b.difficulty
        && a.bend_strength == b.bend_strength
        && a.slide_end_fret == b.slide_end_fret
        && a.unpitched_slide_end_fret == b.unpitched_slide_end_fret
        && a.flags == b.flags
        && a.extended_note_flags == b.extended_note_flags
}

pub fn combine_tech_notes(tech_notes: Vec<EofNote>) -> Vec<EofNote> {
    let mut sorted: Vec<EofNote> = tech_notes;
    sorted.sort_by_key(|x| (x.position, x.difficulty));

    // Reverse fold: combine notes at same position with different bit_flags
    let combined: Vec<EofNote> =
        sorted
            .into_iter()
            .rev()
            .fold(Vec::<EofNote>::new(), |mut acc, current| {
                if let Some(prev) = acc.last() {
                    if is_combinable(&current, prev) {
                        let prev = acc.pop().unwrap();
                        acc.push(combine(current, &prev));
                        return acc;
                    }
                }
                acc.push(current);
                acc
            });
    let combined: Vec<EofNote> = combined.into_iter().rev().collect();

    // Reverse fold: separate notes at same position that can't be at same time
    let separated: Vec<EofNote> =
        combined
            .into_iter()
            .rev()
            .fold(Vec::<EofNote>::new(), |mut acc, a| {
                if let Some(b) = acc.last() {
                    if a.position == b.position && a.difficulty == b.difficulty {
                        let b = acc.pop().unwrap();
                        if can_move(&b) {
                            acc.push(a);
                            acc.push(move_position(b));
                        } else if can_move(&a) {
                            acc.push(b);
                            acc.push(move_position(a));
                        } else {
                            // Neither can move - try pre-bend conversion
                            if a.position == a.actual_note_position
                                && b.position == b.actual_note_position
                            {
                                if a.flags.contains(EofNoteFlag::BEND) {
                                    let a2 = convert_to_pre_bend(move_position(a));
                                    acc.push(b);
                                    acc.push(a2);
                                } else if b.flags.contains(EofNoteFlag::BEND) {
                                    let b2 = convert_to_pre_bend(move_position(b));
                                    acc.push(a);
                                    acc.push(b2);
                                } else {
                                    acc.push(b);
                                    acc.push(a);
                                }
                            } else {
                                acc.push(b);
                                acc.push(a);
                            }
                        }
                        return acc;
                    }
                }
                acc.push(a);
                acc
            });
    separated.into_iter().rev().collect()
}

pub fn get_tech_note_data(tech_notes: &[EofNote]) -> Vec<u8> {
    if tech_notes.is_empty() {
        return vec![];
    }
    let mut buf = Vec::new();
    crate::write_utils::write_notes(&mut buf, tech_notes).unwrap();
    buf
}
