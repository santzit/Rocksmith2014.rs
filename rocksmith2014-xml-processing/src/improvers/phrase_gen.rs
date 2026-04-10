use rocksmith2014_xml::{
    ChordMask, Ebeat, InstrumentalArrangement, Level, NoteMask, Phrase, PhraseIteration, Section,
};

const MIN_PHRASE_SEP: i32 = 2000;

fn content_end(level: &Level) -> i32 {
    let note = level
        .notes
        .iter()
        .map(|n| n.time + n.sustain)
        .max()
        .unwrap_or(0);
    let chord = level
        .chords
        .iter()
        .map(|c| c.time + c.chord_notes.first().map(|cn| cn.sustain).unwrap_or(0))
        .max()
        .unwrap_or(0);
    let hs = level.hand_shapes.last().map(|h| h.end_time).unwrap_or(0);
    note.max(chord).max(hs)
}

fn next_content(level: &Level, from: i32) -> Option<i32> {
    let n = level.notes.iter().find(|x| x.time >= from).map(|x| x.time);
    let c = level.chords.iter().find(|x| x.time >= from).map(|x| x.time);
    let h = level
        .hand_shapes
        .iter()
        .find(|x| x.start_time >= from)
        .map(|x| x.start_time);
    [n, c, h].into_iter().flatten().min()
}

fn end_phrase_time(level: &Level, arr: &InstrumentalArrangement) -> i32 {
    let no_more = content_end(level);
    let candidate = arr
        .phrases
        .iter()
        .position(|p| p.name.eq_ignore_ascii_case("END"))
        .and_then(|idx| {
            arr.phrase_iterations
                .iter()
                .find(|pi| pi.phrase_id as usize == idx)
        })
        .map(|pi| if pi.time < no_more { no_more } else { pi.time })
        .unwrap_or(no_more);

    if candidate != no_more {
        candidate
    } else {
        arr.ebeats
            .iter()
            .find(|b| b.time > no_more)
            .map(|b| b.time)
            .unwrap_or_else(|| (no_more + 100).min(arr.meta.song_length.saturating_sub(100)))
    }
}

/// If `t` falls inside a handshape, return the handshape's end time; otherwise return `t`.
fn adjust_for_handshape(level: &Level, t: i32) -> i32 {
    level
        .hand_shapes
        .iter()
        .find(|h| h.start_time <= t && t < h.end_time)
        .map(|h| h.end_time)
        .unwrap_or(t)
}

/// Returns the end time of the LINK_NEXT chain starting at `note_time` in `level`.
/// The chain end is: last linked note's time + sustain (or + 100 if sustain == 0).
fn note_link_chain_end(level: &Level, note_time: i32) -> i32 {
    let mut cur_time = note_time;
    loop {
        if let Some(cur) = level.notes.iter().find(|n| n.time == cur_time) {
            if cur.mask.contains(NoteMask::LINK_NEXT) {
                let next_time = cur.time + cur.sustain;
                if let Some(next) = level.notes.iter().find(|n| n.time == next_time) {
                    if next.sustain == 0 {
                        return next.time + 100;
                    }
                    cur_time = next_time;
                    continue;
                }
            }
            let end = if cur.sustain == 0 {
                cur.time + 100
            } else {
                cur.time + cur.sustain
            };
            return end;
        } else {
            return cur_time;
        }
    }
}

/// If `t` would fall inside a LINK_NEXT chain (note or chord), pick the chain boundary
/// (chain start or chain end) that is closest to `t`.
fn adjust_for_link_next(level: &Level, t: i32) -> i32 {
    // Check note LINK_NEXT chains
    for note in &level.notes {
        if !note.mask.contains(NoteMask::LINK_NEXT) {
            continue;
        }
        let chain_start = note.time;
        let chain_end = note_link_chain_end(level, chain_start);
        if chain_start <= t && t < chain_end {
            // Pick whichever boundary is closest to original t
            if (t - chain_start).abs() <= (chain_end - t).abs() {
                return chain_start;
            } else {
                return chain_end;
            }
        }
    }
    // Check chord LINK_NEXT chains
    for chord in &level.chords {
        if !chord.mask.contains(ChordMask::LINK_NEXT) {
            continue;
        }
        let chain_start = chord.time;
        // Chain end: chord note sustain + linked note sustain
        let chord_note_sustain = chord.chord_notes.first().map(|cn| cn.sustain).unwrap_or(0);
        let link_target_time = chord.time + chord_note_sustain;
        let chain_end =
            if let Some(linked) = level.notes.iter().find(|n| n.time == link_target_time) {
                if linked.sustain == 0 {
                    linked.time + 100
                } else {
                    linked.time + linked.sustain
                }
            } else {
                link_target_time
            };
        if chain_start <= t && t < chain_end {
            if (t - chain_start).abs() <= (chain_end - t).abs() {
                return chain_start;
            } else {
                return chain_end;
            }
        }
    }
    t
}

/// Generates sections and phrases for the arrangement, replacing existing ones.
pub fn generate_phrases(arr: &mut InstrumentalArrangement) {
    if arr.levels.is_empty() {
        return;
    }
    let level = arr.levels[arr.levels.len() - 1].clone();

    let content_start = match next_content(&level, 0) {
        Some(t) => t,
        None => return,
    };
    let end_time = end_phrase_time(&level, arr);

    arr.phrases.clear();
    arr.phrase_iterations.clear();
    arr.sections.clear();

    // COUNT phrase at first beat
    arr.phrases.push(Phrase {
        name: "COUNT".into(),
        max_difficulty: 0,
        ..Default::default()
    });
    let first_beat = arr.ebeats.first().map(|b| b.time).unwrap_or(0);
    arr.phrase_iterations.push(PhraseIteration {
        time: first_beat,
        end_time: content_start,
        phrase_id: 0,
        ..Default::default()
    });

    // First content phrase/section
    let mut phrase_counter: i32 = 0;
    let mut riff_number: i32 = 1;
    arr.phrases.push(Phrase {
        name: format!("p{}", phrase_counter),
        max_difficulty: 0,
        ..Default::default()
    });
    phrase_counter += 1;
    arr.phrase_iterations.push(PhraseIteration {
        time: content_start,
        end_time,
        phrase_id: 1,
        ..Default::default()
    });
    arr.sections.push(Section {
        name: "riff".into(),
        number: riff_number,
        start_time: content_start,
        end_time,
    });
    riff_number += 1;

    // Walk beats and add phrases every ~8 measures
    let mut measure_counter: i32 = 0;
    let mut last_phrase_time = content_start;
    let beats: Vec<Ebeat> = arr.ebeats.clone();

    for beat in &beats {
        if beat.time < content_start || beat.time >= end_time {
            continue;
        }
        if beat.measure >= 0 {
            measure_counter += 1;
        }
        if measure_counter >= 9 {
            measure_counter = 1;
            let mut t = beat.time;
            // If t falls inside a note's sustain, move boundary back to note start
            if let Some(note_start) = level
                .notes
                .iter()
                .find(|n| n.time < t && n.time + n.sustain > t)
                .map(|n| n.time)
            {
                t = note_start;
            }
            // If t is inside a handshape, move boundary to handshape end
            t = adjust_for_handshape(&level, t);
            // If t would break a LINK_NEXT chain, pick the nearest chain boundary
            t = adjust_for_link_next(&level, t);
            if t - last_phrase_time > MIN_PHRASE_SEP {
                last_phrase_time = t;
                let pid = arr.phrases.len() as u32;
                arr.phrases.push(Phrase {
                    name: format!("p{}", phrase_counter),
                    max_difficulty: 0,
                    ..Default::default()
                });
                phrase_counter += 1;
                arr.phrase_iterations.push(PhraseIteration {
                    time: t,
                    end_time,
                    phrase_id: pid,
                    ..Default::default()
                });
                arr.sections.push(Section {
                    name: "riff".into(),
                    number: riff_number,
                    start_time: t,
                    end_time,
                });
                riff_number += 1;
            }
        }
    }

    // END phrase
    let end_id = arr.phrases.len() as u32;
    arr.phrases.push(Phrase {
        name: "END".into(),
        max_difficulty: 0,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: end_time,
        end_time: arr.meta.song_length,
        phrase_id: end_id,
        ..Default::default()
    });
    arr.sections.push(Section {
        name: "noguitar".into(),
        number: 1,
        start_time: end_time,
        end_time: arr.meta.song_length,
    });
}
