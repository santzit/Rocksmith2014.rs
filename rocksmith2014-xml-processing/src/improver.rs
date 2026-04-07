use std::collections::HashSet;

use regex::Regex;
use rocksmith2014_xml::{Anchor, ArrangementEvent, InstrumentalArrangement, NoteMask};

fn validate_phrase_names(arr: &mut InstrumentalArrangement) {
    let re = Regex::new(r"[^a-zA-Z0-9 _#]").unwrap();
    for phrase in &mut arr.phrases {
        let cleaned = re.replace_all(&phrase.name, "").into_owned();
        phrase.name = cleaned;
    }
}

fn add_ignores(arr: &mut InstrumentalArrangement) {
    let chord_ids: Vec<Vec<i32>> = arr
        .levels
        .iter()
        .map(|l| l.chords.iter().map(|c| c.chord_id).collect())
        .collect();

    for (li, level) in arr.levels.iter_mut().enumerate() {
        for note in &mut level.notes {
            let harmonic = note.mask.contains(NoteMask::HARMONIC);
            if note.fret >= 23 || (note.fret == 7 && note.sustain > 0 && harmonic) {
                note.mask |= NoteMask::IGNORE;
            }
        }
        for (ci, chord) in level.chords.iter_mut().enumerate() {
            let cid = chord_ids[li][ci] as usize;
            if let Some(template) = arr.chord_templates.get(cid) {
                if template.frets.iter().any(|&f| f >= 23) {
                    chord.mask |= rocksmith2014_xml::ChordMask::IGNORE;
                }
            }
        }
    }
}

fn remove_overlapping_bend_values(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        for note in &mut level.notes {
            if !note.bend_values.is_empty() {
                let mut seen = HashSet::new();
                note.bend_values.retain(|bv| seen.insert(bv.time));
            }
        }
        for chord in &mut level.chords {
            for cn in &mut chord.chord_notes {
                if !cn.bend_values.is_empty() {
                    let mut seen = HashSet::new();
                    cn.bend_values.retain(|bv| seen.insert(bv.time));
                }
            }
        }
    }
}

fn fix_link_nexts(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        let count = level.notes.len();
        for i in 0..count {
            if !level.notes[i].mask.contains(NoteMask::LINK_NEXT) {
                continue;
            }
            let (time, sustain, string) = {
                let n = &level.notes[i];
                (n.time, n.sustain, n.string)
            };
            let next_idx = level.notes[i + 1..]
                .iter()
                .position(|n| n.string == string)
                .map(|j| i + 1 + j);
            match next_idx {
                None => {
                    level.notes[i].mask.remove(NoteMask::LINK_NEXT);
                }
                Some(j) if level.notes[j].time - (time + sustain) > 50 => {
                    level.notes[i].mask.remove(NoteMask::LINK_NEXT);
                }
                Some(j) => {
                    let correct = if level.notes[i].slide_to > 0 {
                        level.notes[i].slide_to
                    } else if level.notes[i].slide_unpitch_to > 0 {
                        level.notes[i].slide_unpitch_to
                    } else {
                        level.notes[i].fret
                    };
                    level.notes[j].fret = correct;
                }
            }
        }
    }
}

fn insert_event_sorted(events: &mut Vec<ArrangementEvent>, ev: ArrangementEvent) {
    let pos = events.partition_point(|e| e.time <= ev.time);
    events.insert(pos, ev);
}

fn get_first_note_time(arr: &InstrumentalArrangement) -> Option<i32> {
    let level = arr.levels.first()?;
    let note_t = level.notes.first().map(|n| n.time);
    let chord_t = level.chords.first().map(|c| c.time);
    match (note_t, chord_t) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) | (None, Some(a)) => Some(a),
        (None, None) => None,
    }
}

fn add_crowd_events(arr: &mut InstrumentalArrangement) {
    if arr.events.iter().any(|e| e.code == "E3" || e.code == "D3") {
        return;
    }
    const DELAY: i32 = 600;
    const INTRO_LEN: i32 = 2_500;
    const OUTRO_LEN: i32 = 4_000;
    const FADE_LEN: i32 = 5_000;

    let start = get_first_note_time(arr)
        .map(|t| t + DELAY)
        .unwrap_or(arr.meta.start_beat);
    let end = start + INTRO_LEN;
    insert_event_sorted(
        &mut arr.events,
        ArrangementEvent {
            time: start,
            code: "E3".into(),
        },
    );
    insert_event_sorted(
        &mut arr.events,
        ArrangementEvent {
            time: end,
            code: "E13".into(),
        },
    );

    let audio_end = arr.meta.song_length;
    let outro = audio_end - FADE_LEN - OUTRO_LEN;
    insert_event_sorted(
        &mut arr.events,
        ArrangementEvent {
            time: outro,
            code: "D3".into(),
        },
    );
    insert_event_sorted(
        &mut arr.events,
        ArrangementEvent {
            time: audio_end,
            code: "E13".into(),
        },
    );
}

fn remove_extra_beats(arr: &mut InstrumentalArrangement) {
    let limit = arr.meta.song_length;
    arr.ebeats.retain(|b| b.time <= limit);
}

fn process_chord_names(arr: &mut InstrumentalArrangement) {
    for template in &mut arr.chord_templates {
        if template.display_name.contains("(no name)") {
            let count = template.frets.iter().filter(|&&f| f >= 0).count();
            if count == 2 {
                template.display_name = template
                    .display_name
                    .replace("(no name)", "")
                    .trim()
                    .to_string();
            }
        }
    }
}

fn fix_phrase_start_anchors(arr: &mut InstrumentalArrangement) {
    let phrase_times: Vec<i32> = arr.phrase_iterations.iter().map(|pi| pi.time).collect();
    for level in &mut arr.levels {
        for &pt in &phrase_times {
            if level.anchors.iter().any(|a| a.time == pt) {
                continue;
            }
            if let Some(active) = level.anchors.iter().rev().find(|a| a.time < pt).cloned() {
                let new_anchor = Anchor {
                    time: pt,
                    end_time: active.end_time,
                    fret: active.fret,
                    width: active.width,
                };
                let pos = level.anchors.partition_point(|a| a.time <= pt);
                level.anchors.insert(pos, new_anchor);
            }
        }
    }
}

fn remove_redundant_anchors(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        let mut seen: HashSet<i32> = HashSet::new();
        level.anchors.retain(|a| seen.insert(a.time));
    }
}

/// Applies all improvements to the arrangement.
pub fn apply_all_improvements(arr: &mut InstrumentalArrangement) {
    validate_phrase_names(arr);
    add_ignores(arr);
    fix_link_nexts(arr);
    remove_overlapping_bend_values(arr);
    remove_redundant_anchors(arr);
    add_crowd_events(arr);
    process_chord_names(arr);
    remove_extra_beats(arr);
}

/// Applies the minimum set of improvements needed for export.
pub fn apply_minimum_improvements(arr: &mut InstrumentalArrangement) {
    validate_phrase_names(arr);
    add_ignores(arr);
    remove_overlapping_bend_values(arr);
    fix_phrase_start_anchors(arr);
}
