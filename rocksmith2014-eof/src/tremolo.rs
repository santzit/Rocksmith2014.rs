use crate::types::{EofNote, EofNoteFlag, EofSection};

pub fn create_tremolo_sections(notes: &[EofNote]) -> Vec<EofSection> {
    struct TempSection {
        difficulty: u8,
        prev_index: usize,
        start_time: u32,
        end_time: u32,
    }

    let sections: Vec<TempSection> = notes.iter().enumerate().fold(Vec::new(), |mut acc, (i, note)| {
        if !note.flags.contains(EofNoteFlag::TREMOLO) {
            return acc;
        }
        match acc.last_mut() {
            Some(h) if h.prev_index == i - 1 => {
                h.prev_index = i;
                h.end_time = note.position + note.length;
            }
            _ => {
                acc.push(TempSection {
                    difficulty: note.difficulty,
                    prev_index: i,
                    start_time: note.position,
                    end_time: note.position + note.length,
                });
            }
        }
        acc
    });

    sections.into_iter()
        .map(|s| EofSection::create(s.difficulty, s.start_time, s.end_time, 0))
        .collect()
}
