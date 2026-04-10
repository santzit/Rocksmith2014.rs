use rocksmith2014_xml::InstrumentalArrangement;

fn find_beats(arr: &InstrumentalArrangement, time: i32) -> (i32, i32) {
    let next_idx = arr
        .ebeats
        .iter()
        .position(|b| b.time > time)
        .unwrap_or_else(|| arr.ebeats.len().saturating_sub(1));
    let next_idx = if next_idx == 0 { 1 } else { next_idx };
    let beat1 = arr.ebeats[next_idx - 1].time;
    let beat2 = arr.ebeats[next_idx].time;
    (beat1, beat2)
}

/// Lengthens handshapes that end with a chord (within 10ms of end time).
/// Mirrors HandShapeAdjuster.lengthenHandshapes in the .NET implementation.
pub fn lengthen_handshapes(arr: &mut InstrumentalArrangement) {
    for level_idx in 0..arr.levels.len() {
        let hs_count = arr.levels[level_idx].hand_shapes.len();
        for hs_idx in 0..hs_count {
            let (hs_start, hs_end) = {
                let hs = &arr.levels[level_idx].hand_shapes[hs_idx];
                (hs.start_time, hs.end_time)
            };

            // Find the last chord at or before hs_end within 10ms
            let last_chord_time = arr.levels[level_idx]
                .chords
                .iter()
                .filter(|c| c.time >= hs_start && c.time <= hs_end)
                .map(|c| c.time)
                .max();

            let chord_time = match last_chord_time {
                Some(t) if hs_end - t <= 10 => t,
                _ => continue,
            };

            let (beat1, beat2) = find_beats(arr, chord_time);
            let note16th = (beat2 - beat1) / 4;
            let new_end = hs_end + note16th;

            // Find limiting time (next content, anchor, phrase)
            let next_content = {
                let level = &arr.levels[level_idx];
                let note_t = level
                    .notes
                    .iter()
                    .filter(|n| n.time > chord_time)
                    .map(|n| n.time)
                    .min();
                let chord_t = level
                    .chords
                    .iter()
                    .filter(|c| c.time > chord_time)
                    .map(|c| c.time)
                    .min();
                let hs_t = level
                    .hand_shapes
                    .iter()
                    .filter(|h| h.start_time > chord_time)
                    .map(|h| h.start_time)
                    .min();
                let anchor_t = level
                    .anchors
                    .iter()
                    .filter(|a| a.time > chord_time)
                    .map(|a| a.time)
                    .min();
                let phrase_t = arr
                    .phrase_iterations
                    .iter()
                    .filter(|p| p.time > chord_time)
                    .map(|p| p.time)
                    .min();
                [note_t, chord_t, hs_t, anchor_t, phrase_t]
                    .iter()
                    .filter_map(|x| *x)
                    .min()
            };

            let final_end = match next_content {
                Some(next) if new_end >= next => hs_end + (next - hs_end) / 2,
                _ => new_end,
            };

            arr.levels[level_idx].hand_shapes[hs_idx].end_time = final_end.max(hs_start + 1);
        }
    }
}

/// Shortens handshapes that are too close to the next handshape.
/// Mirrors HandShapeAdjuster.shortenHandshapes in the .NET implementation.
pub fn shorten_handshapes(arr: &mut InstrumentalArrangement) {
    for level_idx in 0..arr.levels.len() {
        let hs_count = arr.levels[level_idx].hand_shapes.len();
        for i in 1..hs_count {
            let (preceding_start, preceding_end, following_start, following_end) = {
                let prev = &arr.levels[level_idx].hand_shapes[i - 1];
                let curr = &arr.levels[level_idx].hand_shapes[i];
                (
                    prev.start_time,
                    prev.end_time,
                    curr.start_time,
                    curr.end_time,
                )
            };

            // Ignore nested handshapes
            if preceding_end >= following_end {
                continue;
            }

            let (beat1, beat2) = find_beats(arr, preceding_end);
            let beat_dur = beat2 - beat1;
            let note32nd = beat_dur / 8;

            let min_distance = if preceding_end - preceding_start <= note32nd {
                beat_dur / 12
            } else {
                note32nd
            };

            let current_distance = following_start - preceding_end;
            if current_distance < min_distance {
                let new_end = {
                    let time = following_start - min_distance;
                    if time <= preceding_start {
                        preceding_start + (preceding_end - preceding_start) / 2
                    } else {
                        time
                    }
                };
                arr.levels[level_idx].hand_shapes[i - 1].end_time = new_end;
            }
        }
    }
}
