use rocksmith2014_xml::InstrumentalArrangement;

fn find_time_of_nth_note_from(arr: &InstrumentalArrangement, from: i32, n: usize) -> Option<i32> {
    // Look at the highest difficulty level for the phrase containing `from`
    let phrase_id = arr
        .phrase_iterations
        .iter()
        .rev()
        .find(|pi| pi.time <= from)
        .map(|pi| pi.phrase_id as usize);

    let max_diff = phrase_id
        .and_then(|pid| arr.phrases.get(pid))
        .map(|p| p.max_difficulty as usize)
        .unwrap_or(0);

    let level = arr.levels.get(max_diff).or_else(|| arr.levels.last())?;

    let mut all_times: Vec<i32> = level
        .notes
        .iter()
        .map(|n| n.time)
        .chain(level.chords.iter().map(|c| c.time))
        .filter(|&t| t >= from)
        .collect();
    all_times.sort_unstable();
    all_times.dedup();

    all_times.get(n.saturating_sub(1)).copied()
}

/// Moves phrase iterations with "mover" prefix to the Nth note/chord from the phrase time.
/// Mirrors PhraseMover.improve in the .NET implementation.
pub fn improve(arr: &mut InstrumentalArrangement) {
    let phrase_count = arr.phrases.len();
    for phrase_idx in 0..phrase_count {
        let phrase_name = arr.phrases[phrase_idx].name.clone();
        if !phrase_name.to_lowercase().starts_with("mover") {
            continue;
        }
        let move_by: usize = phrase_name["mover".len()..]
            .parse()
            .unwrap_or_else(|_| panic!("Unable to parse mover number from phrase name '{phrase_name}'"));

        let pi_indices: Vec<usize> = arr
            .phrase_iterations
            .iter()
            .enumerate()
            .filter(|(_, pi)| pi.phrase_id as usize == phrase_idx)
            .map(|(i, _)| i)
            .collect();

        for pi_idx in pi_indices {
            let pi_time = arr.phrase_iterations[pi_idx].time;

            if let Some(new_time) = find_time_of_nth_note_from(arr, pi_time, move_by) {
                if new_time != pi_time {
                    arr.phrase_iterations[pi_idx].time = new_time;
                    for section in &mut arr.sections {
                        if section.start_time == pi_time {
                            section.start_time = new_time;
                        }
                    }
                    for level in &mut arr.levels {
                        for anchor in &mut level.anchors {
                            if anchor.time == pi_time {
                                anchor.time = new_time;
                            }
                        }
                    }
                }
            }
        }
    }
}
