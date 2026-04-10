use rocksmith2014_xml::InstrumentalArrangement;

fn is_power_chord(tuning: &[i16; 6], frets: &[i8; 6]) -> bool {
    let string_indexes: Vec<usize> = frets
        .iter()
        .enumerate()
        .filter_map(|(i, &f)| if f >= 0 { Some(i) } else { None })
        .collect();

    if string_indexes.len() != 2 {
        return false;
    }
    let (s1, s2) = (string_indexes[0], string_indexes[1]);
    let (f1, f2) = (frets[s1], frets[s2]);

    // Adjacent strings and interval is a fifth
    s1 + 1 == s2 && (tuning[s1] as i32 + f1 as i32) + 2 == (tuning[s2] as i32 + f2 as i32)
}

/// Removes names from double stops (excluding the common power chord shape).
/// Mirrors DoubleStopNameRemover.improve in the .NET implementation.
pub fn improve(tuning: &[i16; 6], arr: &mut InstrumentalArrangement) {
    for template in &mut arr.chord_templates {
        let string_count = template.frets.iter().filter(|&&f| f >= 0).count();
        if string_count != 2 {
            continue;
        }
        if is_power_chord(tuning, &template.frets) {
            continue;
        }
        template.name.clear();
        template.display_name.clear();
    }
}
