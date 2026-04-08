use rocksmith2014_xml::{InstrumentalArrangement, Level, NoteMask};

use crate::{Issue, IssueType};

fn at(kind: IssueType, time: i32) -> Issue {
    Issue::WithTimeCode(kind, time)
}

fn get_noguitar_sections(arr: &InstrumentalArrangement) -> Vec<(i32, i32)> {
    let n = arr.sections.len();
    let mut out = Vec::new();
    for i in 0..n {
        let s = &arr.sections[i];
        let end = if i + 1 < n {
            arr.sections[i + 1].start_time
        } else {
            arr.meta.song_length
        };
        if s.name.starts_with("noguitar") {
            out.push((s.start_time, end));
        }
    }
    out
}

fn get_end_time(arr: &InstrumentalArrangement) -> i32 {
    arr.phrase_iterations
        .last()
        .and_then(|pi| {
            arr.phrases
                .get(pi.phrase_id as usize)
                .filter(|p| p.name.eq_ignore_ascii_case("END"))
                .map(|_| pi.time)
        })
        .unwrap_or(arr.meta.song_length)
}

fn inside_noguitar(sections: &[(i32, i32)], time: i32) -> bool {
    sections.iter().any(|&(s, e)| time >= s && time < e)
}

fn next_note_on_string(
    notes: &[rocksmith2014_xml::Note],
    after_idx: usize,
    string: i8,
) -> Option<&rocksmith2014_xml::Note> {
    notes
        .iter()
        .skip(after_idx + 1)
        .find(|n| n.string == string)
}

fn check_link_next(level: &Level, idx: usize, note: &rocksmith2014_xml::Note) -> Option<Issue> {
    let linked_to_chord = level.chords.iter().any(|c| {
        c.time == note.time + note.sustain
            && !c.chord_notes.is_empty()
            && c.chord_notes.iter().any(|cn| cn.string == note.string)
    });
    if linked_to_chord {
        return Some(at(IssueType::NoteLinkedToChord, note.time));
    }
    match next_note_on_string(&level.notes, idx, note.string) {
        None => Some(at(IssueType::LinkNextMissingTargetNote, note.time)),
        Some(next) if next.time - (note.time + note.sustain) > 1 => {
            Some(at(IssueType::IncorrectLinkNext, note.time))
        }
        Some(next) if note.fret != next.fret => {
            let slide_to = if note.slide_to == -1 {
                note.slide_unpitch_to
            } else {
                note.slide_to
            };
            if slide_to == next.fret {
                None
            } else if slide_to != -1 {
                Some(at(IssueType::LinkNextSlideMismatch, note.time))
            } else {
                Some(at(IssueType::LinkNextFretMismatch, next.time))
            }
        }
        Some(next) if !note.bend_values.is_empty() => {
            let last_step = note.bend_values.last().map_or(0.0, |b| b.step);
            let first_next_step =
                if !next.bend_values.is_empty() && next.bend_values[0].time == next.time {
                    next.bend_values[0].step
                } else {
                    0.0
                };
            if (last_step - first_next_step).abs() > f64::EPSILON {
                Some(at(IssueType::LinkNextBendMismatch, next.time))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn check_overlapping_bends(note: &rocksmith2014_xml::Note) -> Option<Issue> {
    for w in note.bend_values.windows(2) {
        if w[0].time == w[1].time {
            return Some(at(IssueType::OverlappingBendValues, w[1].time));
        }
    }
    None
}

fn check_notes(arr: &InstrumentalArrangement, level: &Level, issues: &mut Vec<Issue>) {
    let ng = get_noguitar_sections(arr);
    let end_time = get_end_time(arr);

    for (i, note) in level.notes.iter().enumerate() {
        let t = note.time;
        let link_next = note.mask.contains(NoteMask::LINK_NEXT);
        let harmonic = note.mask.contains(NoteMask::HARMONIC);
        let pinch = note.mask.contains(NoteMask::PINCH_HARMONIC);
        let ignore = note.mask.contains(NoteMask::IGNORE);
        let is_slide = note.slide_to >= 0;
        let is_unpitch = note.slide_unpitch_to >= 0;
        let is_bend = !note.bend_values.is_empty() || note.max_bend > 0.0;

        if link_next && is_unpitch {
            issues.push(at(IssueType::UnpitchedSlideWithLinkNext, t));
        }
        if harmonic && pinch {
            issues.push(at(IssueType::DoubleHarmonic, t));
        }
        if !ignore && note.fret == 7 && harmonic && note.sustain > 0 {
            issues.push(at(IssueType::SeventhFretHarmonicWithSustain, t));
        }
        if is_bend {
            if harmonic {
                issues.push(at(IssueType::NaturalHarmonicWithBend, t));
            }
            if note.bend_values.iter().all(|bv| bv.step == 0.0) {
                issues.push(at(IssueType::MissingBendValue, t));
            }
            if let Some(ov) = check_overlapping_bends(note) {
                issues.push(ov);
            }
        }
        if arr.tones.iter().any(|tc| tc.time == t) {
            issues.push(at(IssueType::ToneChangeOnNote, t));
        }
        if link_next {
            if let Some(i_issue) = check_link_next(level, i, note) {
                issues.push(i_issue);
            }
        }
        if inside_noguitar(&ng, t) {
            issues.push(at(IssueType::NoteInsideNoguitarSection, t));
        }
        if note.fret > 24 {
            issues.push(at(IssueType::FretNumberMoreThan24, t));
        }
        if note.slide_to > 24 {
            issues.push(at(IssueType::FretNumberMoreThan24, t));
        }
        if note.slide_unpitch_to > 24 {
            issues.push(at(IssueType::FretNumberMoreThan24, t));
        }
        if t > end_time {
            issues.push(at(IssueType::NoteAfterSongEnd, t));
        }
        // Technique notes without sustain
        if (is_bend || is_slide || is_unpitch) && note.sustain == 0 {
            issues.push(at(IssueType::TechniqueNoteWithoutSustain, t));
        }
        // HOPO into same fret
        let is_hopo =
            note.mask.contains(NoteMask::HAMMER_ON) || note.mask.contains(NoteMask::PULL_OFF);
        if is_hopo {
            if let Some(prev) = level.notes[..i]
                .iter()
                .rev()
                .find(|p| p.string == note.string)
            {
                if prev.fret == note.fret {
                    issues.push(at(IssueType::HopoIntoSameNote, t));
                }
            }
        }
    }
}

fn check_phrase_structure(arr: &InstrumentalArrangement, issues: &mut Vec<Issue>) {
    if !arr
        .phrases
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case("END"))
    {
        issues.push(Issue::General(IssueType::NoEndPhrase));
    }
    if let Some(first_pi) = arr.phrase_iterations.first() {
        if let Some(p) = arr.phrases.get(first_pi.phrase_id as usize) {
            if p.max_difficulty > 0 {
                issues.push(Issue::General(IssueType::FirstPhraseNotEmpty));
            }
        }
    }
    if arr.phrases.len() > 100 {
        issues.push(Issue::General(IssueType::MoreThan100Phrases));
    }
}

fn check_crowd_events(arr: &InstrumentalArrangement, issues: &mut Vec<Issue>) {
    let intro = arr.events.iter().find(|e| e.code == "E3");
    let end = arr.events.iter().find(|e| e.code == "E13");
    match (intro, end) {
        (None, _) => {}
        (Some(s), None) => {
            issues.push(Issue::WithTimeCode(
                IssueType::ApplauseEventWithoutEnd,
                s.time,
            ));
        }
        (Some(s), Some(e)) => {
            let re = regex::Regex::new(r"^(e[0-2]|E3|D3)$").unwrap();
            for ev in &arr.events {
                if ev.time > s.time && ev.time < e.time && re.is_match(&ev.code) {
                    issues.push(Issue::WithTimeCode(
                        IssueType::EventBetweenIntroApplause(ev.code.clone()),
                        ev.time,
                    ));
                }
            }
        }
    }
}

/// Runs all checks on the given arrangement and returns a list of issues.
pub fn check_instrumental(arr: &InstrumentalArrangement) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_crowd_events(arr, &mut issues);
    check_phrase_structure(arr, &mut issues);
    if let Some(level) = arr.levels.last() {
        check_notes(arr, level, &mut issues);
    }
    issues
}
