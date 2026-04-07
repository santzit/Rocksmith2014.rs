use crate::types::TimeSignature;
use rocksmith2014_xml::{ArrangementEvent, Ebeat};

pub fn get_closest_beat(beats: &[Ebeat], time: i32) -> usize {
    let index = beats.iter().rposition(|b| b.time <= time);
    match index {
        Some(i) => {
            if let Some(next) = beats.get(i + 1) {
                if (time - beats[i].time).abs() < (time - next.time).abs() {
                    i
                } else {
                    i + 1
                }
            } else {
                i
            }
        }
        None => 0,
    }
}

pub fn try_parse_time_signature(text: &str) -> Option<(u32, u32)> {
    let s = text.strip_prefix("TS:")?;
    let (n, d) = s.split_once('/')?;
    let n: u32 = n.parse().ok()?;
    let d: u32 = d.parse().ok()?;
    if n != 0 && d != 0 {
        Some((n, d))
    } else {
        None
    }
}

pub fn get_time_signatures(events: &[ArrangementEvent]) -> Vec<(i32, TimeSignature)> {
    events
        .iter()
        .filter_map(|e| {
            try_parse_time_signature(&e.code).map(|(n, d)| {
                let ts = match (n, d) {
                    (2, 4) => TimeSignature::TS2_4,
                    (3, 4) => TimeSignature::TS3_4,
                    (4, 4) => TimeSignature::TS4_4,
                    (5, 4) => TimeSignature::TS5_4,
                    (6, 4) => TimeSignature::TS6_4,
                    (a, b) => TimeSignature::Custom(a, b),
                };
                (e.time, ts)
            })
        })
        .collect()
}

pub fn get_beat_count_changes(beats: &[Ebeat]) -> Vec<(i32, usize)> {
    let mut counter = 1usize;
    let mut beat_counts: Vec<(i32, usize)> = Vec::new();

    for beat in beats.iter().rev() {
        if beat.measure < 0 {
            counter += 1;
        } else {
            let same_as_prev = beat_counts
                .last()
                .map(|(_, c)| *c == counter)
                .unwrap_or(false);
            if same_as_prev {
                beat_counts.pop();
            }
            beat_counts.push((beat.time, counter));
            counter = 1;
        }
    }
    beat_counts.reverse();
    beat_counts
}

pub fn infer_time_signatures(beats: &[Ebeat]) -> Vec<(i32, TimeSignature)> {
    get_beat_count_changes(beats)
        .into_iter()
        .map(|(time, beat_count)| {
            let ts = match beat_count {
                2 => TimeSignature::TS2_4,
                3 => TimeSignature::TS3_4,
                4 => TimeSignature::TS4_4,
                5 => TimeSignature::TS5_4,
                6 => TimeSignature::TS6_4,
                9 => TimeSignature::Custom(9, 8),
                12 => TimeSignature::Custom(12, 8),
                v => TimeSignature::Custom(v as u32, 4),
            };
            (time, ts)
        })
        .collect()
}

pub fn is_drop_tuning(tuning: &[i16]) -> bool {
    if tuning.len() < 2 {
        return false;
    }
    let first = tuning[0];
    let expected = tuning[1];
    first == expected - 2 && tuning[1..].iter().all(|&s| s == expected)
}
