use std::collections::{HashMap, HashSet};

use regex::Regex;
use rocksmith2014_xml::{Anchor, ArrangementEvent, ChordMask, InstrumentalArrangement, NoteMask};

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
            // Add ignore to chords that have chord notes with 7th fret harmonic + sustain
            if chord
                .chord_notes
                .iter()
                .any(|cn| cn.fret == 7 && cn.sustain > 0 && cn.mask.contains(NoteMask::HARMONIC))
            {
                chord.mask |= rocksmith2014_xml::ChordMask::IGNORE;
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
    // First pass: remove muted chord notes and collect which templates need updating.
    // We process each chord_id at most once (first occurrence wins).
    let mut template_updates: HashMap<i32, Vec<i8>> = HashMap::new();
    let mut seen_ids: HashSet<i32> = HashSet::new();

    for level in &mut arr.levels {
        for chord in &mut level.chords {
            if seen_ids.contains(&chord.chord_id)
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
            seen_ids.insert(chord.chord_id);
            if muted_strings.is_empty() || muted_strings.len() == chord.chord_notes.len() {
                continue;
            }
            chord.chord_notes.retain(|cn| !cn.mask.contains(NoteMask::FRET_HAND_MUTE));
            template_updates.insert(chord.chord_id, muted_strings);
        }
    }

    // Second pass: update chord templates (no conflict with levels borrow now).
    for (chord_id, muted_strings) in template_updates {
        if let Some(template) = arr.chord_templates.get_mut(chord_id as usize) {
            for s in muted_strings {
                if s >= 0 && (s as usize) < 6 {
                    template.frets[s as usize] = -1;
                    template.fingers[s as usize] = -1;
                }
            }
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

/// Moves anchors that are within 5ms of a note or chord to be exactly at the note/chord time.
/// Does not move if the target note is within 5ms of another note (notes too close together).
/// Mirrors AnchorMover.improve in the .NET implementation.
pub fn move_anchors(arr: &mut InstrumentalArrangement) {
    const MAX_DIFF: i32 = 5;

    for level in &mut arr.levels {
        let event_times: Vec<i32> = level
            .notes
            .iter()
            .map(|n| n.time)
            .chain(level.chords.iter().map(|c| c.time))
            .collect();

        // Note end times for slides/sustained notes
        let note_end_times: Vec<i32> = level
            .notes
            .iter()
            .filter(|n| n.sustain > 0)
            .map(|n| n.time + n.sustain)
            .collect();

        for anchor in &mut level.anchors {
            let closest = event_times
                .iter()
                .min_by_key(|&&t| (t - anchor.time).abs())
                .copied()
                .filter(|&t| (t - anchor.time).abs() <= MAX_DIFF);

            if let Some(target) = closest {
                let too_close = event_times
                    .iter()
                    .chain(note_end_times.iter())
                    .any(|&t| t != target && (t - target).abs() < MAX_DIFF);
                if !too_close {
                    anchor.time = target;
                }
            }
        }
    }
}

/// Removes unnecessary notes with no sustain that follow LINK_NEXT notes or chord slides.
/// Mirrors ArrangementImprover.removeUnnecessaryNotes in the .NET implementation.
pub fn remove_unnecessary_notes(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        let mut to_remove: HashSet<usize> = HashSet::new();

        // Notes without sustain that follow LINK_NEXT notes at the end of their sustain
        for i in 0..level.notes.len() {
            if !level.notes[i].mask.contains(NoteMask::LINK_NEXT) {
                continue;
            }
            let end_time = level.notes[i].time + level.notes[i].sustain;
            let string = level.notes[i].string;
            if let Some(j) = level.notes[i + 1..]
                .iter()
                .position(|n| n.string == string && n.time == end_time && n.sustain == 0)
            {
                to_remove.insert(i + 1 + j);
            }
        }

        // Notes without sustain that follow chord LINK_NEXT (chord slides)
        for chord in &level.chords {
            let is_link_next = chord.mask.contains(ChordMask::LINK_NEXT);
            let has_slide = chord
                .chord_notes
                .iter()
                .any(|cn| cn.slide_to >= 0 || cn.slide_unpitch_to >= 0);
            if !is_link_next && !has_slide {
                continue;
            }
            for cn in &chord.chord_notes {
                let end_time = chord.time + cn.sustain;
                if let Some(j) = level
                    .notes
                    .iter()
                    .position(|n| n.string == cn.string && n.time == end_time && n.sustain == 0)
                {
                    to_remove.insert(j);
                }
            }
        }

        let mut indices: Vec<usize> = to_remove.into_iter().collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            level.notes.remove(idx);
        }
    }
}

/// Removes the HARMONIC mask from notes that also have a slide (slides and harmonics are incompatible).
/// Mirrors HarmonicFixer.improve in the .NET implementation.
pub fn remove_harmonic_mask(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        for note in &mut level.notes {
            if note.mask.contains(NoteMask::HARMONIC)
                && (note.slide_to > 0 || note.slide_unpitch_to > 0)
            {
                note.mask.remove(NoteMask::HARMONIC);
            }
        }
    }
}
