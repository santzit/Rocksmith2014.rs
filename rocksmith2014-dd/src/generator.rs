use crate::types::{GeneratorConfig, LevelCountGeneration};
use crate::utils::{get_allowed_chord_notes, get_note_count, should_start_from_highest_note};
use rocksmith2014_xml::{Chord, ChordNote, InstrumentalArrangement, Level, Note};

fn chord_note_to_note(chord: &Chord, sustain: i32, choose_highest_for_high_strings: bool) -> Option<Note> {
    let cn = if choose_highest_for_high_strings {
        chord.chord_notes.iter().max_by_key(|n| n.string)?
    } else {
        chord.chord_notes.iter().min_by_key(|n| n.string)?
    };

    Some(Note {
        time: chord.time,
        string: cn.string,
        fret: cn.fret,
        sustain,
        vibrato: cn.vibrato,
        slide_to: cn.slide_to,
        slide_unpitch_to: cn.slide_unpitch_to,
        left_hand: cn.left_hand,
        tap: 0,
        pick_direction: 0,
        slap: -1,
        pluck: -1,
        max_bend: 0.0,
        mask: cn.mask,
        bend_values: cn.bend_values.clone(),
    })
}

fn max_chord_strings(chords: &[Chord]) -> usize {
    chords
        .iter()
        .map(|c| c.chord_notes.len())
        .max()
        .unwrap_or(0)
}

pub fn generate_for_arrangement(config: GeneratorConfig, arr: &mut InstrumentalArrangement) {
    if arr.phrases.len() > 1 {
        arr.phrases[1].name = "p0".to_string();
    }

    let level_count = match config.level_count_generation {
        LevelCountGeneration::Constant(v) => v.max(1),
        LevelCountGeneration::Simple => 2,
        LevelCountGeneration::MlModel => 2,
    };

    let Some(hardest) = arr.levels.first().cloned() else {
        return;
    };

    let mut generated = Vec::with_capacity(level_count);
    let max_strings = max_chord_strings(&hardest.chords);

    for diff in 0..level_count {
        if diff == level_count - 1 {
            let mut level = hardest.clone();
            level.difficulty = diff as i8;
            generated.push(level);
            continue;
        }

        let diff_percent = (diff + 1) as f64 / level_count as f64;
        let allowed_notes = get_allowed_chord_notes(diff_percent, max_strings);
        let sustain_for_notes = if diff_percent < 0.2 {
            0
        } else {
            800
        };

        let mut level = Level {
            difficulty: diff as i8,
            anchors: hardest.anchors.clone(),
            hand_shapes: hardest.hand_shapes.clone(),
            notes: hardest.notes.clone(),
            chords: Vec::new(),
        };

        for chord in &hardest.chords {
            let template = arr
                .chord_templates
                .get(chord.chord_id as usize)
                .cloned()
                .unwrap_or_default();
            let note_count = get_note_count(&template);

            if allowed_notes <= 1 && !chord.chord_notes.is_empty() {
                let choose_highest = should_start_from_highest_note(note_count, &template);
                if let Some(note) = chord_note_to_note(chord, sustain_for_notes, choose_highest) {
                    level.notes.push(note);
                }
            } else if allowed_notes >= note_count {
                level.chords.push(chord.clone());
            } else if let Some(ChordNote {
                string,
                fret,
                sustain,
                vibrato,
                slide_to,
                slide_unpitch_to,
                left_hand,
                bend_values,
                mask,
            }) = chord.chord_notes.first()
            {
                level.notes.push(Note {
                    time: chord.time,
                    string: *string,
                    fret: *fret,
                    sustain: if diff_percent < 0.2 { 0 } else { *sustain },
                    vibrato: *vibrato,
                    slide_to: *slide_to,
                    slide_unpitch_to: *slide_unpitch_to,
                    left_hand: *left_hand,
                    tap: 0,
                    pick_direction: 0,
                    slap: -1,
                    pluck: -1,
                    max_bend: 0.0,
                    mask: *mask,
                    bend_values: bend_values.clone(),
                });
            }
        }

        generated.push(level);
    }

    arr.levels = generated;
}
