use rocksmith2014_xml::{Anchor, ChordMask, InstrumentalArrangement, NoteMask};

/// Adds LinkNext to chords missing the attribute when a chord note has it.
/// Also fixes varying sustains: sets all chord note sustains to the max.
/// Mirrors EOFFixes.fixChordNotes in the .NET implementation.
pub fn fix_chord_notes(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        for chord in &mut level.chords {
            if chord
                .chord_notes
                .iter()
                .any(|cn| cn.mask.contains(NoteMask::LINK_NEXT))
            {
                chord.mask |= ChordMask::LINK_NEXT;
            }
            if !chord.chord_notes.is_empty() {
                let max_sustain = chord
                    .chord_notes
                    .iter()
                    .map(|cn| cn.sustain)
                    .max()
                    .unwrap_or(0);
                for cn in &mut chord.chord_notes {
                    cn.sustain = max_sustain;
                }
            }
        }
    }
}

/// Removes invalid chord note linknexts when no target note is found within 1ms.
/// Mirrors EOFFixes.removeInvalidChordNoteLinkNexts in the .NET implementation.
pub fn remove_invalid_chord_note_link_nexts(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        let note_string_times: Vec<(i8, i32)> =
            level.notes.iter().map(|n| (n.string, n.time)).collect();
        for chord in &mut level.chords {
            if !chord.mask.contains(ChordMask::LINK_NEXT) {
                continue;
            }
            for cn in &mut chord.chord_notes {
                if !cn.mask.contains(NoteMask::LINK_NEXT) {
                    continue;
                }
                let target_time = chord.time + cn.sustain;
                let found = note_string_times
                    .iter()
                    .any(|&(s, t)| s == cn.string && (t - target_time).abs() <= 1);
                if !found {
                    cn.mask.remove(NoteMask::LINK_NEXT);
                }
            }
        }
    }
}

/// Fixes incorrect crowd events (uppercase E0/E1/E2 → lowercase e0/e1/e2).
/// Mirrors EOFFixes.fixCrowdEvents in the .NET implementation.
pub fn fix_crowd_events(arr: &mut InstrumentalArrangement) {
    for ev in &mut arr.events {
        match ev.code.as_str() {
            "E0" => ev.code = "e0".to_string(),
            "E1" => ev.code = "e1".to_string(),
            "E2" => ev.code = "e2".to_string(),
            _ => {}
        }
    }
}

/// Fixes chord slide handshape end times to match chord note sustain.
/// Mirrors EOFFixes.fixChordSlideHandshapes in the .NET implementation.
pub fn fix_chord_slide_handshapes(arr: &mut InstrumentalArrangement) {
    for level in &mut arr.levels {
        let chord_data: Vec<(i32, i32)> = level
            .chords
            .iter()
            .filter(|c| c.mask.contains(ChordMask::LINK_NEXT))
            .filter_map(|c| {
                let max_sustain = c
                    .chord_notes
                    .iter()
                    .filter(|cn| cn.slide_to >= 0)
                    .map(|cn| cn.sustain)
                    .max();
                max_sustain.map(|s| (c.time, c.time + s))
            })
            .collect();
        for hs in &mut level.hand_shapes {
            if let Some(&(_, end_time)) = chord_data.iter().find(|&&(t, _)| t == hs.start_time) {
                hs.end_time = end_time;
            }
        }
    }
}

/// Ensures anchors exist at phrase start times.
/// Anchors that fall within a phrase section are moved to the phrase start.
/// Phrase sections without an anchor at the start get a copy of the preceding active anchor.
/// The last phrase iteration (END phrase) is excluded from processing.
/// Mirrors EOFFixes.fixPhraseStartAnchors in the .NET implementation.
pub fn fix_phrase_start_anchors(arr: &mut InstrumentalArrangement) {
    if arr.phrase_iterations.len() < 2 {
        return;
    }
    // Exclude the last phrase iteration (END phrase marker)
    let phrase_times: Vec<i32> = arr.phrase_iterations[..arr.phrase_iterations.len() - 1]
        .iter()
        .map(|pi| pi.time)
        .collect();

    for level in &mut arr.levels {
        let n = phrase_times.len();
        for i in 0..n {
            let pt = phrase_times[i];
            let next_pt = if i + 1 < n {
                phrase_times[i + 1]
            } else {
                i32::MAX
            };

            if level.anchors.iter().any(|a| a.time == pt) {
                continue;
            }

            // Find first anchor in [pt, next_pt): move it to pt
            if let Some(pos) = level
                .anchors
                .iter()
                .position(|a| a.time >= pt && a.time < next_pt)
            {
                level.anchors[pos].time = pt;
            } else {
                // No anchor in this phrase section — copy from last anchor before pt
                if let Some(active) = level.anchors.iter().rev().find(|a| a.time < pt).cloned() {
                    let new_anchor = Anchor { time: pt, ..active };
                    let ins_pos = level.anchors.partition_point(|a| a.time <= pt);
                    level.anchors.insert(ins_pos, new_anchor);
                }
            }
        }
    }
}
