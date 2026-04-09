use std::collections::HashSet;

use regex::Regex;
use rocksmith2014_xml::{Anchor, ArrangementEvent, InstrumentalArrangement, NoteMask};

/// Returns the time of the first note or chord across all levels.
pub fn get_first_note_time(arr: &InstrumentalArrangement) -> Option<i32> {
    arr.levels
        .iter()
        .flat_map(|level| {
            let note_t = level.notes.iter().map(|n| n.time);
            let chord_t = level.chords.iter().map(|c| c.time);
            note_t.chain(chord_t)
        })
        .min()
}

pub fn validate_phrase_names(arr: &mut InstrumentalArrangement) {
    let re = Regex::new(r"[^a-zA-Z0-9 _#]").unwrap();
    for phrase in &mut arr.phrases {
        let cleaned = re.replace_all(&phrase.name, "").into_owned();
        phrase.name = cleaned;
    }
}

pub fn add_ignores(arr: &mut InstrumentalArrangement) {
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

pub fn remove_overlapping_bend_values(arr: &mut InstrumentalArrangement) {
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

pub fn fix_link_nexts(arr: &mut InstrumentalArrangement) {
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

pub fn add_crowd_events(arr: &mut InstrumentalArrangement) {
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

/// Removes beats that come after the audio has ended.
/// Mirrors ExtraBeatRemover.improve in the .NET implementation.
pub fn remove_extra_beats(arr: &mut InstrumentalArrangement) {
    let audio_end = arr.meta.song_length;
    if arr.ebeats.len() < 2 {
        return;
    }
    while arr.ebeats.len() >= 2 && arr.ebeats[arr.ebeats.len() - 2].time > audio_end {
        arr.ebeats.pop();
    }
    if arr.ebeats.len() < 2 {
        return;
    }
    let last_idx = arr.ebeats.len() - 1;
    let penultimate_time = arr.ebeats[last_idx - 1].time;
    let last_time = arr.ebeats[last_idx].time;
    if audio_end - penultimate_time <= last_time - audio_end {
        arr.ebeats.pop();
    }
    if let Some(last) = arr.ebeats.last_mut() {
        last.time = audio_end;
    }
}

/// Processes chord names: replaces "min" with "m", handles -arp, -nop, CONV suffixes.
/// Mirrors ChordNameProcessor.improve in the .NET implementation.
pub fn process_chord_names(arr: &mut InstrumentalArrangement) {
    fn empty_or_else(s: &str, f: impl Fn(&str) -> String) -> String {
        if s.trim().is_empty() { String::new() } else { f(s) }
    }
    for template in &mut arr.chord_templates {
        template.name = empty_or_else(&template.name, |name| {
            name.replace("min", "m")
                .replace("CONV", "")
                .replace("-nop", "")
                .replace("-arp", "")
        });
        template.display_name = empty_or_else(&template.display_name, |dname| {
            dname.replace("min", "m").replace("CONV", "-arp")
        });
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

/// Removes anchors identical (same fret and width) to the previous anchor,
/// unless the anchor is at a phrase iteration time.
/// Mirrors BasicFixes.removeRedundantAnchors in the .NET implementation.
pub fn remove_redundant_anchors(arr: &mut InstrumentalArrangement) {
    let phrase_times: HashSet<i32> = arr.phrase_iterations.iter().map(|pi| pi.time).collect();
    for level in &mut arr.levels {
        if level.anchors.len() <= 1 {
            continue;
        }
        let anchors = std::mem::take(&mut level.anchors);
        let mut result = Vec::with_capacity(anchors.len());
        result.push(anchors[0].clone());
        for i in 1..anchors.len() {
            let prev = &anchors[i - 1];
            let curr = &anchors[i];
            let identical = prev.fret == curr.fret && prev.width == curr.width;
            if !identical || phrase_times.contains(&curr.time) {
                result.push(curr.clone());
            }
        }
        level.anchors = result;
    }
}

/// Removes fret-hand-muted notes from chords that also contain normal notes.
/// Mirrors BasicFixes.removeMutedNotesFromChords in the .NET implementation.
pub fn remove_muted_notes_from_chords(arr: &mut InstrumentalArrangement) {
    let mut fixed_chord_templates: HashSet<i32> = HashSet::new();
    for level in &mut arr.levels {
        for chord in &mut level.chords {
            if fixed_chord_templates.contains(&chord.chord_id)
                || chord.chord_notes.is_empty()
                || chord.mask.contains(rocksmith2014_xml::ChordMask::FRET_HAND_MUTE)
            {
                continue;
            }
            let muted_strings: Vec<i8> = chord
                .chord_notes
                .iter()
                .filter(|cn| cn.mask.contains(NoteMask::FRET_HAND_MUTE))
                .map(|cn| cn.string)
                .collect();
            if muted_strings.is_empty() || muted_strings.len() == chord.chord_notes.len() {
                continue;
            }
            chord.chord_notes.retain(|cn| !cn.mask.contains(NoteMask::FRET_HAND_MUTE));
            if let Some(template) = arr.chord_templates.get_mut(chord.chord_id as usize) {
                for &s in &muted_strings {
                    if s >= 0 && (s as usize) < 6 {
                        template.frets[s as usize] = -1;
                        template.fingers[s as usize] = -1;
                    }
                }
            }
            fixed_chord_templates.insert(chord.chord_id);
        }
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
