use std::collections::HashMap;

use rocksmith2014_xml::{Anchor, ChordMask, ChordNote, ChordTemplate, HandShape, Note, NoteMask};
use rocksmith2014_xml_extension::XmlEntity;

pub type NoteScore = i32;

// ---- Scoring ----

fn get_subdivision(start: i32, end: i32, time: i32) -> i32 {
    let dist = (end - start) as f64;
    if dist <= 0.0 {
        return 2;
    }
    let pos = (time - start) as f64;
    let mid = dist / 2.0;
    let p = if pos - mid > 10.0 { pos - mid } else { pos };
    if p <= 0.0 {
        return 20;
    }
    (dist / p).round() as i32
}

fn subdivision_in_measure(phrase_end: i32, beats: &[rocksmith2014_xml::Ebeat], time: i32) -> i32 {
    let measure = beats.iter().rev().find(|b| b.time < time && b.measure >= 0);
    match measure {
        None => 2,
        Some(first) => {
            let end = beats
                .iter()
                .find(|b| b.time > time && b.measure >= 0)
                .map(|b| b.time)
                .unwrap_or(phrase_end);
            get_subdivision(first.time, end, time)
        }
    }
}

const PHRASE_DIVS: [i32; 9] = [0, 3, 2, 3, 1, 3, 2, 3, 4];

fn division_in_phrase(start: i32, end: i32, time: i32) -> i32 {
    let len = (end - start) as f64;
    if len <= 0.0 {
        return 0;
    }
    let div_len = len / PHRASE_DIVS.len() as f64;
    let pos = (time - start) as f64;
    let idx = (0..PHRASE_DIVS.len())
        .find(|&i| pos >= div_len * i as f64 && pos < div_len * (i + 1) as f64)
        .unwrap_or(PHRASE_DIVS.len() - 1);
    PHRASE_DIVS[idx]
}

fn technique_penalty(entity: &XmlEntity) -> i32 {
    match entity {
        XmlEntity::Note(n) => {
            if n.mask.contains(NoteMask::FRET_HAND_MUTE) {
                20
            } else if n.max_bend > 0.0 || !n.bend_values.is_empty() {
                10
            } else {
                0
            }
        }
        XmlEntity::Chord(c) => {
            if c.mask.contains(ChordMask::FRET_HAND_MUTE) {
                20
            } else {
                0
            }
        }
    }
}

pub fn score_entity(
    phrase_start: i32,
    phrase_end: i32,
    beats: &[rocksmith2014_xml::Ebeat],
    time: i32,
    entity: &XmlEntity,
) -> NoteScore {
    let div = division_in_phrase(phrase_start, phrase_end, time);
    let pen = technique_penalty(entity);

    let beat1 = beats.iter().rev().find(|b| b.time <= time);
    let beat2 = beats.iter().find(|b| b.time >= time);

    let sub = match (beat1, beat2) {
        (None, _) => 20,
        (Some(b1), Some(b2)) => {
            if time == b1.time {
                if b1.measure >= 0 {
                    0
                } else {
                    subdivision_in_measure(phrase_end, beats, time)
                }
            } else {
                10 * get_subdivision(b1.time, b2.time, time)
            }
        }
        (Some(b1), None) => 10 * get_subdivision(b1.time, phrase_end, time),
    };

    pen + div + sub
}

/// Creates score map: sorted list of (score, low, high) difficulty ranges.
pub fn create_score_map(scores: &[(i32, NoteScore)], total: usize) -> Vec<(NoteScore, f64, f64)> {
    let mut groups: std::collections::BTreeMap<NoteScore, usize> =
        std::collections::BTreeMap::new();
    for &(_, s) in scores {
        *groups.entry(s).or_insert(0) += 1;
    }
    let mut result = Vec::new();
    let mut low = 0.0f64;
    for (score, count) in &groups {
        let high = low + *count as f64 / total as f64;
        result.push((*score, low, high));
        low = high;
    }
    result
}

// ---- Entity choosing ----

pub fn get_note_count(template: &ChordTemplate) -> usize {
    template.frets.iter().filter(|&&f| f >= 0).count()
}

pub fn allowed_chord_notes(diff_percent: f64, max: usize) -> usize {
    if max == 0 {
        0
    } else {
        (diff_percent * max as f64).ceil() as usize
    }
}

fn should_start_from_highest(total: usize, template: &ChordTemplate) -> bool {
    let fr = &template.frets;
    (total == 3 && fr[5] >= 0 && fr[4] >= 0 && fr[3] >= 0)
        || (total == 2 && fr[5] >= 0 && fr[4] >= 0)
}

fn should_exclude(
    diff: f64,
    score: NoteScore,
    score_map: &[(NoteScore, f64, f64)],
    current: &mut HashMap<NoteScore, usize>,
    notes_with_score: &HashMap<NoteScore, usize>,
) -> bool {
    if let Some(&(_, low, high)) = score_map.iter().find(|(s, _, _)| *s == score) {
        if diff < low {
            return true;
        }
        if diff >= low && diff < high {
            let total = *notes_with_score.get(&score).unwrap_or(&0);
            let cur = *current.get(&score).unwrap_or(&0);
            let frac = (diff - low) / (high - low);
            let min_n = if low == 0.0 { 1 } else { 0 };
            let allowed = ((total as f64 * frac).round() as usize).max(min_n);
            return (cur + 1) > allowed;
        }
        false
    } else {
        false
    }
}

pub fn prune_note(diff: f64, note: &mut Note) {
    if diff <= 0.2 {
        if note.bend_values.is_empty() && note.max_bend == 0.0 {
            note.sustain = 0;
        }
        note.mask.remove(
            NoteMask::LINK_NEXT
                | NoteMask::HAMMER_ON
                | NoteMask::PULL_OFF
                | NoteMask::TREMOLO
                | NoteMask::PICK_DIRECTION,
        );
        note.vibrato = 0;
        note.slide_to = -1;
        note.slide_unpitch_to = -1;
    } else if diff <= 0.35 && note.mask.contains(NoteMask::TREMOLO) {
        note.mask.remove(NoteMask::TREMOLO);
    } else if diff <= 0.45 && note.vibrato != 0 {
        note.vibrato = 0;
    }
    if diff <= 0.60 && note.mask.contains(NoteMask::PINCH_HARMONIC) {
        note.mask.remove(NoteMask::PINCH_HARMONIC);
    }
}

pub fn prune_chord_note(diff: f64, cn: &mut ChordNote) {
    if diff <= 0.2 {
        if cn.bend_values.is_empty() {
            cn.sustain = 0;
        }
        cn.mask.remove(
            NoteMask::LINK_NEXT | NoteMask::HAMMER_ON | NoteMask::PULL_OFF | NoteMask::TREMOLO,
        );
        cn.vibrato = 0;
        cn.slide_to = -1;
        cn.slide_unpitch_to = -1;
    } else if diff <= 0.35 && cn.mask.contains(NoteMask::TREMOLO) {
        cn.mask.remove(NoteMask::TREMOLO);
    } else if diff <= 0.45 && cn.vibrato != 0 {
        cn.vibrato = 0;
    }
    if diff <= 0.60 && cn.mask.contains(NoteMask::PINCH_HARMONIC) {
        cn.mask.remove(NoteMask::PINCH_HARMONIC);
    }
}

fn note_from_chord(
    diff: f64,
    chord: &rocksmith2014_xml::Chord,
    template: &ChordTemplate,
    total: usize,
) -> Note {
    let from_highest = should_start_from_highest(total, template);
    if !chord.chord_notes.is_empty() {
        let idx = if from_highest {
            chord.chord_notes.len() - 1
        } else {
            0
        };
        let cn = &chord.chord_notes[idx];
        let mut note = Note {
            time: chord.time,
            string: cn.string,
            fret: cn.fret,
            sustain: cn.sustain,
            vibrato: cn.vibrato,
            slide_to: cn.slide_to,
            slide_unpitch_to: cn.slide_unpitch_to,
            left_hand: -1,
            mask: cn.mask,
            bend_values: cn.bend_values.clone(),
            ..Default::default()
        };
        prune_note(diff, &mut note);
        note
    } else {
        let find_fn: fn(&[i8]) -> Option<(usize, &i8)> = if from_highest {
            |frets: &[i8]| frets.iter().enumerate().rev().find(|(_, &f)| f >= 0)
        } else {
            |frets: &[i8]| frets.iter().enumerate().find(|(_, &f)| f >= 0)
        };
        let (string, &fret) = find_fn(&template.frets).unwrap_or((0, &0));
        let mut note = Note {
            time: chord.time,
            string: string as i8,
            fret,
            ..Default::default()
        };
        if chord.mask.contains(ChordMask::FRET_HAND_MUTE) {
            note.mask |= NoteMask::FRET_HAND_MUTE;
        }
        if chord.mask.contains(ChordMask::PALM_MUTE) {
            note.mask |= NoteMask::PALM_MUTE;
        }
        if chord.mask.contains(ChordMask::ACCENT) {
            note.mask |= NoteMask::ACCENT;
        }
        if chord.mask.contains(ChordMask::IGNORE) {
            note.mask |= NoteMask::IGNORE;
        }
        prune_note(diff, &mut note);
        note
    }
}

/// Chooses entities for a difficulty level.
pub fn choose_entities(
    diff: f64,
    score_map: &[(NoteScore, f64, f64)],
    time_to_score: &HashMap<i32, NoteScore>,
    notes_with_score: &HashMap<NoteScore, usize>,
    templates: &[ChordTemplate],
    max_chord_notes: usize,
    entities: &[XmlEntity],
) -> (Vec<Note>, Vec<rocksmith2014_xml::Chord>) {
    let mut notes = Vec::new();
    let mut chords = Vec::new();
    let mut current: HashMap<NoteScore, usize> = HashMap::new();
    let allowed = allowed_chord_notes(diff, max_chord_notes);

    for entity in entities {
        let time = entity.time_code();
        let score = *time_to_score.get(&time).unwrap_or(&0);
        if should_exclude(diff, score, score_map, &mut current, notes_with_score) {
            continue;
        }
        *current.entry(score).or_insert(0) += 1;

        match entity {
            XmlEntity::Note(orig) => {
                let mut note = orig.clone();
                prune_note(diff, &mut note);
                notes.push(note);
            }
            XmlEntity::Chord(chord) => {
                let tid = chord.chord_id as usize;
                let template = match templates.get(tid) {
                    Some(t) => t,
                    None => {
                        chords.push(chord.clone());
                        continue;
                    }
                };
                let note_count = get_note_count(template);
                if allowed <= 1 {
                    notes.push(note_from_chord(diff, chord, template, note_count));
                } else if allowed >= note_count {
                    let mut c = chord.clone();
                    for cn in &mut c.chord_notes {
                        prune_chord_note(diff, cn);
                    }
                    chords.push(c);
                } else {
                    let mut c = chord.clone();
                    let to_remove = c.chord_notes.len().saturating_sub(allowed);
                    for _ in 0..to_remove {
                        c.chord_notes.pop();
                    }
                    for cn in &mut c.chord_notes {
                        prune_chord_note(diff, cn);
                    }
                    chords.push(c);
                }
            }
        }
    }
    (notes, chords)
}

const ERR_MARGIN: i32 = 3;

/// Chooses anchors relevant to the level entities.
pub fn choose_anchors(
    entities: &[XmlEntity],
    anchors: &[Anchor],
    start: i32,
    end: i32,
) -> Vec<Anchor> {
    anchors
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            let next = anchors.get(i + 1).map(|x| x.time).unwrap_or(end);
            a.time == start
                || entities.iter().any(|e| {
                    let t = e.time_code();
                    let s = e.sustain();
                    (t + ERR_MARGIN >= a.time && t < next)
                        || (t + s + ERR_MARGIN >= a.time && t + s < next)
                })
        })
        .map(|(_, a)| a.clone())
        .collect()
}

/// Chooses hand shapes relevant to the level entities.
pub fn choose_hand_shapes(
    diff: f64,
    level_entities: &[XmlEntity],
    max_chord_notes: usize,
    hand_shapes: &[HandShape],
) -> Vec<HandShape> {
    let allowed = allowed_chord_notes(diff, max_chord_notes);
    if allowed <= 1 {
        return Vec::new();
    }
    hand_shapes
        .iter()
        .filter(|hs| {
            level_entities.iter().any(|e| {
                let t = e.time_code();
                t >= hs.start_time && t < hs.end_time
            })
        })
        .cloned()
        .collect()
}
