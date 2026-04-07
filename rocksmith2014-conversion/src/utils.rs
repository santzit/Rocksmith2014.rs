use rocksmith2014_xml::{Anchor as XmlAnchor, PhraseIteration as XmlPhraseIteration, Section as XmlSection};
use rocksmith2014_sng::FingerPrint as SngFingerPrint;

/// Converts seconds (f32) to milliseconds (i32), rounding.
pub fn sec_to_ms(s: f32) -> i32 {
    (s as f64 * 1000.0).round() as i32
}

/// Converts milliseconds (i32) to seconds (f32).
pub fn ms_to_sec(ms: i32) -> f32 {
    ms as f32 / 1000.0
}

/// Converts a null-terminated byte slice to a String.
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// Copies a string into a fixed-length byte array (null-padded).
pub fn string_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut result = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

/// Finds the phrase iteration index for a given time (inclusive match).
pub fn find_phrase_iteration_id(time: i32, phrase_iterations: &[XmlPhraseIteration]) -> usize {
    if phrase_iterations.is_empty() {
        return 0;
    }
    let mut id = phrase_iterations.len() - 1;
    while id > 0 {
        let pi_time = phrase_iterations[id].time;
        if pi_time == time || pi_time < time {
            break;
        }
        id -= 1;
    }
    id
}

/// Finds the phrase iteration for beats (exclusive — beat on same time as PI belongs to previous PI).
pub fn find_beat_phrase_iteration_id(time: i32, phrase_iterations: &[XmlPhraseIteration]) -> usize {
    if phrase_iterations.is_empty() {
        return 0;
    }
    let mut id = phrase_iterations.len() - 1;
    while id > 0 {
        if phrase_iterations[id].time < time {
            break;
        }
        id -= 1;
    }
    id
}

/// Finds the section index for a given time.
pub fn find_section_id(time: i32, sections: &[XmlSection]) -> usize {
    if sections.is_empty() {
        return 0;
    }
    let mut id = sections.len() - 1;
    while id > 0 && sections[id].start_time > time {
        id -= 1;
    }
    id
}

/// Finds the anchor for the note at the given time code.
pub fn find_anchor<'a>(time: i32, anchors: &'a [XmlAnchor]) -> &'a XmlAnchor {
    let mut index = anchors.len() as isize - 1;
    while index >= 0 {
        if anchors[index as usize].time <= time {
            return &anchors[index as usize];
        }
        index -= 1;
    }
    panic!("No anchor found for note at time {:.3}", ms_to_sec(time))
}

/// Finds the first and last note indices within [start_time, end_time).
pub fn find_first_and_last_time(
    note_times: &[i32],
    start_time: i32,
    end_time: i32,
) -> Option<(usize, usize)> {
    if note_times.is_empty() {
        return None;
    }
    // Optimization: start searching from mid-point if possible
    let start_hint = {
        let mid = note_times.len() / 2;
        if note_times[mid] < start_time { mid } else { 0 }
    };

    let first = note_times[start_hint..].iter().position(|&t| t >= start_time)?;
    let first = first + start_hint;

    if note_times[first] >= end_time {
        return None;
    }

    let last = note_times.iter().rposition(|&t| t < end_time)?;
    Some((first, last))
}

/// Finds the finger print ID for a note at the given time (seconds).
pub fn find_finger_print_id(time: f32, finger_prints: &[SngFingerPrint]) -> i16 {
    for (i, fp) in finger_prints.iter().enumerate() {
        if time >= fp.start_time && time < fp.end_time {
            return i as i16;
        }
    }
    -1
}
