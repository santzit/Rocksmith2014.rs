/// Converts a tuning pitch (Hz) into cents relative to A440.
pub fn tuning_pitch_to_cents(pitch: f64) -> f64 {
    (1200.0 * (pitch / 440.0).log2()).round()
}

/// Converts a cent offset back into a tuning pitch in Hz.
pub fn cents_to_tuning_pitch(cents: f64) -> f64 {
    let raw = 440.0 * 2.0_f64.powf(cents / 1200.0);
    (raw * 100.0).round() / 100.0
}

const ROOTS: [&str; 12] = [
    "E", "F", "F#", "G", "Ab", "A", "Bb", "B", "C", "C#", "D", "Eb",
];
const STANDARD_TUNING_OFFSETS: [i32; 6] = [0, 5, 10, 3, 7, 0];

fn get_string_note_name(use_flats: bool, string_index: usize, string_tuning: i16) -> &'static str {
    let m =
        ((string_tuning as i32 + STANDARD_TUNING_OFFSETS[string_index]).rem_euclid(12)) as usize;
    let n = ROOTS[m];
    if use_flats {
        match n {
            "F#" => "Gb",
            "C#" => "Db",
            _ => n,
        }
    } else {
        n
    }
}

fn all_same(strings: &[i16]) -> bool {
    strings.iter().all(|&x| x == strings[0])
}

fn is_double_drop(first: i16, strings: &[i16; 6]) -> bool {
    first == strings[1] - 2 && first == strings[5] && all_same(&strings[1..5])
}

/// The name of a tuning, either a translatable preset or a custom string.
#[derive(Debug, Clone, PartialEq)]
pub enum TuningName {
    /// A named preset tuning (e.g. "Standard", "Drop") with format arguments.
    Translatable(String, Vec<String>),
    /// A custom tuning identified by its note names (e.g. "DADGAD").
    Custom(String),
}

/// Returns a name for the given 6-string tuning.
pub fn get_tuning_name(tuning: &[i16; 6]) -> TuningName {
    let first = tuning[0];

    // Standard: all strings same offset, and that offset is in [-10, +2]
    if all_same(tuning) && first > -11 && first < 3 {
        let n = get_string_note_name(false, 0, first);
        return TuningName::Translatable("Standard".to_string(), vec![n.to_string()]);
    }

    // Drop: string 0 is two semitones below string 1, strings 1-5 all same
    if first == tuning[1] - 2 && all_same(&tuning[1..]) {
        let n1 = get_string_note_name(true, 0, first);
        let n2 = get_string_note_name(true, 0, tuning[1]);
        let root = if first < -2 {
            format!("{} ", n2)
        } else {
            String::new()
        };
        return TuningName::Translatable("Drop".to_string(), vec![root, n1.to_string()]);
    }

    // Double drop: string 0 and string 5 are the same, two below string 1, strings 1-4 all same
    if is_double_drop(first, tuning) {
        let n = get_string_note_name(true, 0, first);
        return TuningName::Translatable("Double Drop".to_string(), vec![n.to_string()]);
    }

    // Named open tunings
    match tuning {
        [-2, 0, 0, -1, -2, -2] => {
            return TuningName::Translatable("OpenTuning".to_string(), vec!["D".to_string()])
        }
        [0, 0, 2, 2, 2, 0] => {
            return TuningName::Translatable("OpenTuning".to_string(), vec!["A".to_string()])
        }
        [-2, -2, 0, 0, 0, -2] => {
            return TuningName::Translatable("OpenTuning".to_string(), vec!["G".to_string()])
        }
        [0, 2, 2, 1, 0, 0] => {
            return TuningName::Translatable("OpenTuning".to_string(), vec!["E".to_string()])
        }
        _ => {}
    }

    // Custom tuning: concatenate note names for each string
    let name: String = tuning
        .iter()
        .enumerate()
        .map(|(i, &t)| get_string_note_name(true, i, t))
        .collect();
    TuningName::Custom(name)
}
