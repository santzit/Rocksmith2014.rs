//! Helper functions for EOF project generation.

use regex::Regex;
use rocksmith2014_xml::Ebeat;

use crate::types::EofTimeSignature;

/// Returns the index of the beat closest to the given time.
pub fn get_closest_beat(beats: &[Ebeat], time: i32) -> usize {
    let idx = beats.iter().rposition(|b| b.time <= time);
    let next = idx.and_then(|i| beats.get(i + 1));
    match (idx, next) {
        (Some(i), Some(nb)) => {
            if (time - beats[i].time).abs() < (time - nb.time).abs() {
                i
            } else {
                i + 1
            }
        }
        (Some(i), None) => i,
        _ => 0,
    }
}

/// Attempts to parse a time signature from an event string like `"TS:4/4"`.
pub fn try_parse_time_signature(text: &str) -> Option<(u32, u32)> {
    let re = Regex::new(r"TS:(\d+)/(\d+)").unwrap();
    let caps = re.captures(text)?;
    let n: u32 = caps[1].parse().ok()?;
    let d: u32 = caps[2].parse().ok()?;
    if n == 0 || d == 0 {
        None
    } else {
        Some((n, d))
    }
}

/// Infers time signatures from a beat sequence by counting beats per measure.
///
/// Returns a list of `(time, EofTimeSignature)` pairs.
pub fn infer_time_signatures(
    beats: impl IntoIterator<Item = Ebeat>,
) -> Vec<(i32, EofTimeSignature)> {
    let beats: Vec<Ebeat> = beats.into_iter().collect();
    if beats.is_empty() {
        return Vec::new();
    }

    // Walk in reverse, counting non-measure beats (measure = -1 is a "sub-beat")
    let mut beat_counts: Vec<(i32, i32)> = Vec::new();
    let mut counter = 1i32;
    let mut prev_count = -1i32;

    for beat in beats.iter().rev() {
        if beat.measure < 0 {
            counter += 1;
        } else {
            // If the count hasn't changed, replace the stored time with this earlier one
            if prev_count == counter {
                if let Some(last) = beat_counts.last_mut() {
                    last.0 = beat.time;
                }
            } else {
                beat_counts.push((beat.time, counter));
                prev_count = counter;
            }
            counter = 1;
        }
    }

    beat_counts.reverse();

    beat_counts
        .into_iter()
        .map(|(time, count)| {
            let ts = match count {
                2 => EofTimeSignature::Ts2_4,
                3 => EofTimeSignature::Ts3_4,
                4 => EofTimeSignature::Ts4_4,
                5 => EofTimeSignature::Ts5_4,
                6 => EofTimeSignature::Ts6_4,
                9 => EofTimeSignature::Custom(9, 8),
                12 => EofTimeSignature::Custom(12, 8),
                v => EofTimeSignature::Custom(v as u32, 4),
            };
            (time, ts)
        })
        .collect()
}
