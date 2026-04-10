use rocksmith2014_xml::{InstrumentalArrangement, Level, NoteMask};

use crate::types::{Issue, IssueType};

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

pub fn check_notes(arr: &InstrumentalArrangement, level: &Level) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_notes_inner(arr, level, &mut issues);
    issues
}

pub fn check_crowd_events(arr: &InstrumentalArrangement) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_crowd_events_inner(arr, &mut issues);
    issues
}

pub fn check_phrases(arr: &InstrumentalArrangement) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_phrase_structure(arr, &mut issues);
    issues
}

pub fn check_chords(arr: &InstrumentalArrangement, level: &Level) -> Vec<Issue> {
    let mut issues = Vec::new();
    let ng = get_noguitar_sections(arr);
    let end_time = get_end_time(arr);
    for chord in &level.chords {
        let t = chord.time;
        if arr.tones.iter().any(|tc| tc.time == t) {
            issues.push(at(IssueType::ToneChangeOnNote, t));
        }
        if inside_noguitar(&ng, t) {
            issues.push(at(IssueType::NoteInsideNoguitarSection, t));
        }
        if t >= end_time {
            issues.push(at(IssueType::NoteAfterSongEnd, t));
        }
        let link_next = chord.mask.contains(rocksmith2014_xml::ChordMask::LINK_NEXT);
        if link_next && chord.chord_notes.is_empty() {
            issues.push(at(IssueType::MissingLinkNextChordNotes, t));
        } else if link_next && !chord.chord_notes.iter().any(|cn| cn.mask.contains(NoteMask::LINK_NEXT)) {
            issues.push(at(IssueType::MissingLinkNextChordNotes, t));
        }
        for cn in &chord.chord_notes {
            let cn_link = cn.mask.contains(NoteMask::LINK_NEXT);
            let harmonic = cn.mask.contains(NoteMask::HARMONIC);
            let pinch = cn.mask.contains(NoteMask::PINCH_HARMONIC);
            if harmonic && pinch { issues.push(at(IssueType::DoubleHarmonic, t)); }
            if cn.fret == 7 && harmonic && cn.sustain > 0 { issues.push(at(IssueType::SeventhFretHarmonicWithSustain, t)); }
            let is_bend = !cn.bend_values.is_empty();
            if is_bend && cn.bend_values.iter().all(|bv| bv.step == 0.0) {
                issues.push(at(IssueType::MissingBendValue, t));
            }
            for w in cn.bend_values.windows(2) {
                if w[0].time == w[1].time { issues.push(at(IssueType::OverlappingBendValues, w[1].time)); }
            }
            if cn_link && cn.slide_unpitch_to >= 0 { issues.push(at(IssueType::UnpitchedSlideWithLinkNext, t)); }
            if cn_link {
                let next = level.notes.iter().find(|n| n.string == cn.string && n.time > t);
                match next {
                    None => issues.push(at(IssueType::LinkNextMissingTargetNote, t)),
                    Some(n) if n.time - (t + cn.sustain) > 1 => issues.push(at(IssueType::IncorrectLinkNext, t)),
                    Some(n) if cn.slide_to >= 0 && cn.slide_to != n.fret => issues.push(at(IssueType::LinkNextSlideMismatch, t)),
                    Some(n) if !cn.bend_values.is_empty() => {
                        let last = cn.bend_values.last().map_or(0.0, |b| b.step);
                        let first_next = if !n.bend_values.is_empty() && n.bend_values[0].time == n.time { n.bend_values[0].step } else { 0.0 };
                        if (last - first_next).abs() > f64::EPSILON { issues.push(at(IssueType::LinkNextBendMismatch, n.time)); }
                    }
                    _ => {}
                }
            }
            // TechniqueNoteWithoutSustain for chord notes
            let is_slide = cn.slide_to >= 0;
            let is_unpitch = cn.slide_unpitch_to >= 0;
            let is_vibrato = cn.vibrato > 0;
            let is_tremolo = cn.mask.contains(NoteMask::TREMOLO);
            if (is_slide || is_unpitch || is_bend || is_vibrato || is_tremolo) && cn.sustain < 5 {
                issues.push(at(IssueType::TechniqueNoteWithoutSustain, t));
            }
        }
        // MutedStringInNonMutedChord
        if !chord.chord_notes.is_empty() {
            let all_muted = chord.chord_notes.iter().all(|cn| cn.mask.contains(NoteMask::FRET_HAND_MUTE));
            if !all_muted && chord.chord_notes.iter().any(|cn| cn.mask.contains(NoteMask::FRET_HAND_MUTE)) {
                issues.push(at(IssueType::MutedStringInNonMutedChord, t));
            }
        }
        // Chord fingering checks
        if let Some(template) = arr.chord_templates.get(chord.chord_id as usize) {
            let fingers = &template.fingers;
            let uses_thumb = fingers.iter().any(|&f| f == 0);
            let has_barre = {
                let fret_arr = &template.frets;
                let active_fingers: Vec<i8> = fingers.iter().copied().filter(|&f| f > 0).collect();
                let has_repeated_finger = active_fingers
                    .iter()
                    .any(|&f| active_fingers.iter().filter(|&&g| g == f).count() > 1);
                let has_open_unplayed = fret_arr
                    .iter()
                    .zip(fingers.iter())
                    .any(|(&fr, &fi)| fi < 0 && fr == 0);
                let has_non_open_active = (0..6usize)
                    .any(|i| fingers[i] > 0 && fret_arr[i] > 0);
                has_repeated_finger && has_open_unplayed && has_non_open_active
            };
            // PossiblyWrongChordFingering - first finger not on lowest fret
            if !uses_thumb && !chord.chord_notes.is_empty() {
                let active_frets: Vec<(i8, i8)> = (0..6usize)
                    .filter(|&i| fingers[i] > 0)
                    .map(|i| (fingers[i], template.frets[i]))
                    .collect();
                if !active_frets.is_empty() {
                    let min_fret = active_frets.iter().map(|&(_, f)| f).filter(|&f| f > 0).min();
                    if let Some(mf) = min_fret {
                        let min_finger_at_min_fret = active_frets.iter().filter(|&&(_, f)| f == mf).map(|&(fi, _)| fi).min();
                        let overall_min_finger = active_frets.iter().map(|&(fi, _)| fi).min();
                        if min_finger_at_min_fret != overall_min_finger {
                            issues.push(at(IssueType::PossiblyWrongChordFingering, t));
                        }
                    }
                }
            }
            // BarreOverOpenStrings
            if !uses_thumb && has_barre {
                issues.push(at(IssueType::BarreOverOpenStrings, t));
            }
        }
        // InvalidBassArrangementString  
        if arr.meta.arrangement_properties.path_bass > 0 {
            for cn in &chord.chord_notes {
                if cn.string > 3 { issues.push(at(IssueType::InvalidBassArrangementString, t)); break; }
            }
        }
        // PositionShiftIntoPullOff for chords with pull-off notes
        if chord.chord_notes.iter().any(|cn| cn.mask.contains(NoteMask::PULL_OFF)) {
            let anchor = level.anchors.iter().rev().find(|a| a.time <= t);
            let prev_anchor = level.anchors.iter().rev().find(|a| a.time < t);
            if let (Some(a), Some(pa)) = (anchor, prev_anchor) {
                if a.fret != pa.fret { issues.push(at(IssueType::PositionShiftIntoPullOff, t)); }
            }
        }
    }
    issues
}

pub fn check_handshapes(arr: &InstrumentalArrangement, level: &Level) -> Vec<Issue> {
    let mut issues = Vec::new();
    for hs in &level.hand_shapes {
        let anchor = level.anchors.iter().rev().find(|a| a.time <= hs.start_time);
        if let Some(a) = anchor {
            if let Some(template) = arr.chord_templates.get(hs.chord_id as usize) {
                let uses_thumb = template.fingers.iter().any(|&f| f == 0);
                if !uses_thumb {
                    // Collect active (finger > 0) (finger, fret) pairs
                    let active: Vec<(i8, i8)> = (0..6usize)
                        .filter(|&i| template.fingers[i] > 0)
                        .map(|i| (template.fingers[i], template.frets[i]))
                        .collect();
                    if !active.is_empty() {
                        let min_finger = active.iter().map(|&(f, _)| f).min().unwrap();
                        // Expected anchor fret = min fret used by the lowest finger - (min_finger - 1)
                        // This is the position finger 1 would occupy for this chord shape.
                        let min_fret_at_min_finger = active
                            .iter()
                            .filter(|&&(f, _)| f == min_finger)
                            .map(|&(_, r)| r)
                            .filter(|&r| r > 0)
                            .min();
                        if let Some(mfr) = min_fret_at_min_finger {
                            let expected_anchor = mfr - (min_finger - 1);
                            if a.fret != expected_anchor {
                                issues.push(at(IssueType::FingeringAnchorMismatch, hs.start_time));
                            }
                        }
                    }
                }
            }
        }
    }
    issues
}

pub fn check_anchors(arr: &InstrumentalArrangement, level: &Level) -> Vec<Issue> {
    let mut issues = Vec::new();
    let phrase_times: Vec<i32> = arr.phrase_iterations.iter().map(|pi| pi.time).collect();
    let will_be_moved: Vec<i32> = arr.phrase_iterations.iter()
        .filter(|pi| arr.phrases.get(pi.phrase_id as usize)
            .map_or(false, |p| p.name.to_lowercase().starts_with("mover")))
        .map(|pi| pi.time)
        .collect();
    for anchor in &level.anchors {
        let t = anchor.time;
        // AnchorInsideHandShape
        let inside = level.hand_shapes.iter().any(|hs| t > hs.start_time && t < hs.end_time);
        if inside {
            let at_phrase_boundary = phrase_times.iter().any(|&pt| t == pt);
            if at_phrase_boundary {
                let mover = will_be_moved.iter().any(|&mt| {
                    level.hand_shapes.iter().any(|hs| hs.start_time < mt && hs.end_time > mt && t >= hs.start_time && t < hs.end_time)
                });
                if !mover { issues.push(at(IssueType::AnchorInsideHandShapeAtPhraseBoundary, t)); }
            } else {
                issues.push(at(IssueType::AnchorInsideHandShape, t));
            }
        }
        // AnchorCloseToUnpitchedSlide
        for note in &level.notes {
            if note.slide_unpitch_to >= 0 && note.sustain > 0 {
                let slide_end = note.time + note.sustain;
                if (t - slide_end).abs() <= 5 && t > note.time {
                    let mover = will_be_moved.iter().any(|&mt| t == mt || (t > mt - 5 && t <= mt));
                    if !mover { issues.push(at(IssueType::AnchorCloseToUnpitchedSlide, t)); }
                }
            }
        }
    }
    issues
}

fn check_notes_inner(arr: &InstrumentalArrangement, level: &Level, issues: &mut Vec<Issue>) {
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
        if t > end_time {
            issues.push(at(IssueType::NoteAfterSongEnd, t));
        }
        // Technique notes without sustain
        if (is_bend || is_slide || is_unpitch) && note.sustain == 0 {
            issues.push(at(IssueType::TechniqueNoteWithoutSustain, t));
        }
        if arr.meta.arrangement_properties.path_bass > 0 && note.string > 3 {
            issues.push(at(IssueType::InvalidBassArrangementString, t));
        }
        // HOPO into same fret
        let is_pull_off = note.mask.contains(NoteMask::PULL_OFF);
        let is_hammer_on = note.mask.contains(NoteMask::HAMMER_ON);
        if is_pull_off || is_hammer_on {
            // Find the effective previous fret on the same string.
            // For HAMMER_ON: only check if note[i-1] is on the same string (hammer-on from nowhere otherwise).
            // For PULL_OFF: also consider chords and slides.
            let effective_prev_fret = if is_hammer_on {
                // Only check immediate predecessor
                if i > 0 && level.notes[i - 1].string == note.string {
                    Some(level.notes[i - 1].fret)
                } else {
                    None
                }
            } else {
                // PULL_OFF: find most recent note or chord on same string
                let last_note = level.notes[..i]
                    .iter()
                    .rev()
                    .find(|p| p.string == note.string);
                let last_chord = level
                    .chords
                    .iter()
                    .rev()
                    .find(|c| c.time < t);
                // Pick whichever is more recent
                match (last_note, last_chord) {
                    (Some(n), Some(c)) if c.time > n.time => {
                        // Chord is more recent; get fret from chord_notes or template
                        let cn_fret = c.chord_notes.iter()
                            .find(|cn| cn.string == note.string)
                            .map(|cn| if cn.slide_to >= 0 { cn.slide_to as i8 } else { cn.fret });
                        if let Some(f) = cn_fret {
                            Some(f)
                        } else {
                            arr.chord_templates.get(c.chord_id as usize)
                                .map(|tmpl| tmpl.frets[note.string as usize])
                        }
                    }
                    (Some(n), _) => {
                        Some(if n.slide_to >= 0 { n.slide_to as i8 } else { n.fret })
                    }
                    (None, Some(c)) => {
                        let cn_fret = c.chord_notes.iter()
                            .find(|cn| cn.string == note.string)
                            .map(|cn| if cn.slide_to >= 0 { cn.slide_to as i8 } else { cn.fret });
                        if let Some(f) = cn_fret {
                            Some(f)
                        } else {
                            arr.chord_templates.get(c.chord_id as usize)
                                .map(|tmpl| tmpl.frets[note.string as usize])
                        }
                    }
                    (None, None) => None,
                }
            };
            if let Some(pf) = effective_prev_fret {
                if pf == note.fret {
                    issues.push(at(IssueType::HopoIntoSameNote, t));
                }
            }
        }
        // PositionShiftIntoPullOff for notes
        if is_pull_off && note.fret > 0 {
            // Check if there is an anchor starting exactly at this note's time
            if let Some(cur_anchor) = level.anchors.iter().find(|a| a.time == t) {
                // Find the previous anchor (any anchor with time < t)
                if let Some(prev_anchor) = level.anchors.iter().rev().find(|a| a.time < t) {
                    if cur_anchor.fret != prev_anchor.fret {
                        issues.push(at(IssueType::PositionShiftIntoPullOff, t));
                    }
                }
            }
        }
        // PhraseChangeOnLinkNextNote
        if link_next && note.sustain > 0 {
            let sustain_end = t + note.sustain;
            for pi in &arr.phrase_iterations {
                if pi.time > t && pi.time < sustain_end {
                    // Ignore mover phrases
                    let is_mover = arr.phrases.get(pi.phrase_id as usize)
                        .map_or(false, |p| p.name.to_lowercase().starts_with("mover"));
                    if !is_mover {
                        issues.push(at(IssueType::PhraseChangeOnLinkNextNote, t));
                        break;
                    }
                }
            }
        }
        // FingerChangeDuringSlide
        if is_slide && note.sustain > 0 {
            let slide_end = t + note.sustain;
            let start_anchor = level.anchors.iter().rev().find(|a| a.time <= t);
            let end_anchor = level.anchors.iter().find(|a| a.time >= slide_end);
            if let (Some(sa), Some(ea)) = (start_anchor, end_anchor) {
                // Skip slides from low position where finger cannot be determined from anchor:
                // when sa.fret == 1 and note is at most 1 fret above anchor (finger 1 or 2)
                let offset = note.fret - sa.fret;
                if sa.fret > 1 || offset > 1 {
                    let finger_start = offset + 1;
                    let finger_end = note.slide_to as i8 - ea.fret + 1;
                    if finger_start != finger_end {
                        issues.push(at(IssueType::FingerChangeDuringSlide, t));
                    }
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
    if arr.phrase_iterations.len() > 100 {
        issues.push(Issue::General(IssueType::MoreThan100Phrases));
    }
}

fn check_crowd_events_inner(arr: &InstrumentalArrangement, issues: &mut Vec<Issue>) {
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
    check_crowd_events_inner(arr, &mut issues);
    check_phrase_structure(arr, &mut issues);
    if let Some(level) = arr.levels.last() {
        check_notes_inner(arr, level, &mut issues);
    }
    issues
}
