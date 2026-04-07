use rocksmith2014_xml::{
    InstrumentalArrangement, Note, ChordNote, Chord, ChordTemplate, BendValue, NoteMask, ChordMask,
};
use crate::types::{EofNote, EofNoteFlag, EofExtendedNoteFlag};

fn get_bit_flag(string: i8) -> u8 {
    1u8 << (string as u8)
}

fn note_is_fret_hand_mute(mask: NoteMask) -> bool { mask.contains(NoteMask::FRET_HAND_MUTE) }
fn note_is_palm_mute(mask: NoteMask) -> bool { mask.contains(NoteMask::PALM_MUTE) }
fn note_is_harmonic(mask: NoteMask) -> bool { mask.contains(NoteMask::HARMONIC) }
fn note_is_pinch_harmonic(mask: NoteMask) -> bool { mask.contains(NoteMask::PINCH_HARMONIC) }
fn note_is_hammer_on(mask: NoteMask) -> bool { mask.contains(NoteMask::HAMMER_ON) }
fn note_is_pull_off(mask: NoteMask) -> bool { mask.contains(NoteMask::PULL_OFF) }
fn note_is_vibrato(vibrato: i8) -> bool { vibrato != 0 }
fn note_is_tap(tap: i8) -> bool { tap != 0 }
fn note_is_link_next(mask: NoteMask) -> bool { mask.contains(NoteMask::LINK_NEXT) }
fn note_is_accent(mask: NoteMask) -> bool { mask.contains(NoteMask::ACCENT) }
fn note_is_tremolo(mask: NoteMask) -> bool { mask.contains(NoteMask::TREMOLO) }
fn note_is_ignore(mask: NoteMask) -> bool { mask.contains(NoteMask::IGNORE) }

#[derive(Clone)]
struct NoteData {
    string: i8,
    fret: i8,
    sustain: i32,
    time: i32,
    slide_to: i8,
    slide_unpitch_to: i8,
    vibrato: i8,
    tap: i8,
    slap: i8,
    pluck: i8,
    bend_values: Vec<BendValue>,
    mask: NoteMask,
}

impl NoteData {
    fn from_note(n: &Note) -> Self {
        NoteData {
            string: n.string,
            fret: n.fret,
            sustain: n.sustain,
            time: n.time,
            slide_to: n.slide_to,
            slide_unpitch_to: n.slide_unpitch_to,
            vibrato: n.vibrato,
            tap: n.tap,
            slap: n.slap,
            pluck: n.pluck,
            bend_values: n.bend_values.clone(),
            mask: n.mask,
        }
    }

    fn from_chord_note(cn: &ChordNote, chord_time: i32) -> Self {
        NoteData {
            string: cn.string,
            fret: cn.fret,
            sustain: cn.sustain,
            time: chord_time,
            slide_to: cn.slide_to,
            slide_unpitch_to: cn.slide_unpitch_to,
            vibrato: cn.vibrato,
            tap: 0,
            slap: 0,
            pluck: 0,
            bend_values: cn.bend_values.clone(),
            mask: cn.mask,
        }
    }

    fn is_fret_hand_mute(&self) -> bool { note_is_fret_hand_mute(self.mask) }
    fn is_palm_mute(&self) -> bool { note_is_palm_mute(self.mask) }
    fn is_harmonic(&self) -> bool { note_is_harmonic(self.mask) }
    fn is_pinch_harmonic(&self) -> bool { note_is_pinch_harmonic(self.mask) }
    fn is_hammer_on(&self) -> bool { note_is_hammer_on(self.mask) }
    fn is_pull_off(&self) -> bool { note_is_pull_off(self.mask) }
    fn is_hopo(&self) -> bool { self.is_hammer_on() || self.is_pull_off() }
    fn is_vibrato(&self) -> bool { note_is_vibrato(self.vibrato) }
    fn is_slap(&self) -> bool { self.slap != 0 }
    fn is_pluck(&self) -> bool { self.pluck != 0 }
    fn is_link_next(&self) -> bool { note_is_link_next(self.mask) }
    fn is_accent(&self) -> bool { note_is_accent(self.mask) }
    fn is_tremolo(&self) -> bool { note_is_tremolo(self.mask) }
    fn is_ignore(&self) -> bool { note_is_ignore(self.mask) }
    fn is_slide(&self) -> bool { self.slide_to > 0 }
    fn is_unpitched_slide(&self) -> bool { self.slide_unpitch_to > 0 }
    fn is_bend(&self) -> bool { !self.bend_values.is_empty() }
    fn is_tap(&self) -> bool { note_is_tap(self.tap) }
}

#[derive(Clone)]
struct ChordData {
    template: ChordTemplate,
    chord_id: i16,
    handshape_id: i32,
    is_full_panel: bool,
    is_first_in_hand_shape: bool,
    is_link_next: bool,
    fingering: Vec<u8>,
}

struct NoteGroup {
    chord: Option<ChordData>,
    time: u32,
    difficulty: u8,
    notes: Vec<NoteData>,
}

fn convert_template_fingering(template: &ChordTemplate, string_index: usize) -> u8 {
    match template.fingers[string_index] {
        0 => 5, // Thumb
        f if f < 0 => 0,
        f => f as u8,
    }
}

fn frets_from_template(notes: &[NoteData], ct: &ChordTemplate) -> Vec<u8> {
    ct.frets.iter().enumerate()
        .filter_map(|(i, &f)| {
            if f < 0 {
                None
            } else {
                let is_muted = notes.iter().any(|n| n.string == i as i8 && n.is_fret_hand_mute());
                Some(if is_muted { 128u8 | f as u8 } else { f as u8 })
            }
        })
        .collect()
}

fn fingering_from_template(ct: &ChordTemplate) -> Vec<u8> {
    ct.frets.iter().enumerate()
        .filter_map(|(i, &f)| if f >= 0 { Some(i) } else { None })
        .map(|i| convert_template_fingering(ct, i))
        .collect()
}

fn bit_flag_from_template(ct: &ChordTemplate) -> u8 {
    ct.frets.iter().enumerate()
        .filter(|(_, &f)| f >= 0)
        .fold(0u8, |acc, (i, _)| acc | get_bit_flag(i as i8))
}

fn notes_from_template(c: &Chord, template: &ChordTemplate) -> Vec<NoteData> {
    template.frets.iter().enumerate()
        .filter_map(|(i, &fret)| {
            if fret < 0 { return None; }
            let mut mask = NoteMask::empty();
            if c.mask.contains(ChordMask::ACCENT) { mask |= NoteMask::ACCENT; }
            if c.mask.contains(ChordMask::FRET_HAND_MUTE) { mask |= NoteMask::FRET_HAND_MUTE; }
            if c.mask.contains(ChordMask::PALM_MUTE) { mask |= NoteMask::PALM_MUTE; }
            if c.mask.contains(ChordMask::IGNORE) { mask |= NoteMask::IGNORE; }
            Some(NoteData {
                string: i as i8,
                fret,
                sustain: 0,
                time: c.time,
                slide_to: -1,
                slide_unpitch_to: -1,
                vibrato: 0,
                tap: 0,
                slap: 0,
                pluck: 0,
                bend_values: vec![],
                mask,
            })
        })
        .collect()
}

fn get_note_flags(ext_flag: EofExtendedNoteFlag, note: &NoteData) -> EofNoteFlag {
    let mut flags = EofNoteFlag::empty();
    if note.is_fret_hand_mute() { flags |= EofNoteFlag::STRING_MUTE; }
    if note.is_palm_mute() { flags |= EofNoteFlag::PALM_MUTE; }
    if note.is_harmonic() { flags |= EofNoteFlag::HARMONIC; }
    if note.is_pinch_harmonic() { flags |= EofNoteFlag::P_HARMONIC; }
    if note.is_hammer_on() { flags |= EofNoteFlag::HO; }
    if note.is_pull_off() { flags |= EofNoteFlag::PO; }
    if note.is_hopo() {
        flags |= EofNoteFlag::HOPO;
        flags |= EofNoteFlag::F_HOPO;
    }
    if note.is_tap() { flags |= EofNoteFlag::TAP; }
    if note.is_vibrato() { flags |= EofNoteFlag::VIBRATO; }
    if note.is_slap() { flags |= EofNoteFlag::SLAP; }
    if note.is_pluck() { flags |= EofNoteFlag::POP; }
    if note.is_link_next() { flags |= EofNoteFlag::LINKNEXT; }
    if note.is_accent() { flags |= EofNoteFlag::ACCENT; }
    if note.is_tremolo() { flags |= EofNoteFlag::TREMOLO; }
    if note.is_unpitched_slide() { flags |= EofNoteFlag::UNPITCH_SLIDE; }
    if note.is_slide() {
        flags |= EofNoteFlag::RS_NOTATION;
        if note.slide_to > note.fret {
            flags |= EofNoteFlag::SLIDE_UP;
        } else {
            flags |= EofNoteFlag::SLIDE_DOWN;
        }
    }
    if !ext_flag.is_empty() { flags |= EofNoteFlag::EXTENDED_FLAGS; }
    flags
}

fn get_extended_note_flags(chord_data: Option<&ChordData>, note: &NoteData) -> EofExtendedNoteFlag {
    let sustain_flag_needed = note.sustain > 0
        && chord_data.map(|c| {
            !c.is_link_next
                && !note.is_vibrato()
                && !note.is_bend()
                && !note.is_tremolo()
                && !note.is_slide()
                && !note.is_unpitched_slide()
        }).unwrap_or(false);

    let mut flags = EofExtendedNoteFlag::empty();
    if note.is_ignore() { flags |= EofExtendedNoteFlag::IGNORE; }
    if sustain_flag_needed { flags |= EofExtendedNoteFlag::SUSTAIN; }
    flags
}

fn convert_bend_value(step: f64) -> u8 {
    let is_quarter = step.ceil() != step;
    if is_quarter {
        (step * 2.0) as u8 | 128u8
    } else {
        step as u8
    }
}

fn create_note_groups(inst: &InstrumentalArrangement) -> Vec<NoteGroup> {
    let mut groups: Vec<NoteGroup> = Vec::new();

    for (diff, level) in inst.levels.iter().enumerate() {
        let diff = diff as u8;

        // Ghost chords: hand shapes where neither a note nor chord starts at that time
        for hs in &level.hand_shapes {
            let has_note_at_start = level.notes.iter().any(|n| n.time == hs.start_time)
                || level.chords.iter().any(|c| c.time == hs.start_time);
            if !has_note_at_start {
                let ct = &inst.chord_templates[hs.chord_id as usize];
                let fingering = fingering_from_template(ct);
                let chord_data = ChordData {
                    template: ct.clone(),
                    chord_id: hs.chord_id as i16,
                    handshape_id: -1,
                    is_full_panel: false,
                    is_first_in_hand_shape: true,
                    is_link_next: false,
                    fingering,
                };
                groups.push(NoteGroup {
                    chord: Some(chord_data),
                    time: hs.start_time as u32,
                    difficulty: diff,
                    notes: vec![],
                });
            }
        }

        // Individual notes (group by time)
        let mut note_by_time: std::collections::BTreeMap<i32, Vec<NoteData>> = std::collections::BTreeMap::new();
        for n in &level.notes {
            note_by_time.entry(n.time).or_default().push(NoteData::from_note(n));
        }
        for (time, mut notes) in note_by_time {
            notes.sort_by_key(|n| n.string);
            groups.push(NoteGroup {
                chord: None,
                time: time as u32,
                difficulty: diff,
                notes,
            });
        }

        // Chords
        for c in &level.chords {
            let template = &inst.chord_templates[c.chord_id as usize];
            let notes: Vec<NoteData> = if !c.chord_notes.is_empty() {
                let mut ns: Vec<NoteData> = c.chord_notes.iter()
                    .map(|cn| NoteData::from_chord_note(cn, c.time))
                    .collect();
                ns.sort_by_key(|n| n.string);
                ns
            } else {
                let mut ns = notes_from_template(c, template);
                ns.sort_by_key(|n| n.string);
                ns
            };

            let handshape_id = level.hand_shapes.iter().position(|hs| {
                c.time >= hs.start_time && c.time < hs.end_time
            }).map(|i| i as i32).unwrap_or(-1);

            let handshape_start_time = if handshape_id >= 0 {
                level.hand_shapes[handshape_id as usize].start_time
            } else {
                -1
            };

            let fingering: Vec<u8> = notes.iter()
                .map(|n| convert_template_fingering(template, n.string as usize))
                .collect();

            let chord_data = ChordData {
                template: template.clone(),
                chord_id: c.chord_id as i16,
                handshape_id,
                is_full_panel: !c.chord_notes.is_empty() && !c.mask.contains(ChordMask::HIGH_DENSITY),
                is_first_in_hand_shape: c.time == handshape_start_time,
                is_link_next: c.mask.contains(ChordMask::LINK_NEXT),
                fingering,
            };

            groups.push(NoteGroup {
                chord: Some(chord_data),
                time: c.time as u32,
                difficulty: diff,
                notes,
            });
        }
    }

    groups.sort_by_key(|g| (g.time, g.difficulty as u32));
    groups
}

pub fn convert_notes(inst: &InstrumentalArrangement) -> (Vec<EofNote>, Vec<Vec<u8>>, Vec<Vec<EofNote>>) {
    let note_groups = create_note_groups(inst);
    let n = note_groups.len();
    let mut eof_notes = Vec::with_capacity(n);
    let mut fingerings = Vec::with_capacity(n);
    let mut tech_notes_list: Vec<Vec<EofNote>> = Vec::with_capacity(n);

    for (index, group) in note_groups.iter().enumerate() {
        let NoteGroup { chord: chord_opt, notes, time, difficulty: diff } = group;

        if notes.is_empty() {
            // Ghost chord
            let chord = chord_opt.as_ref().unwrap();
            let bit_flag = bit_flag_from_template(&chord.template);
            let frets = frets_from_template(notes, &chord.template);

            let eof_note = EofNote {
                difficulty: *diff,
                chord_name: chord.template.chord_name.clone(),
                bit_flag,
                ghost_bit_flag: bit_flag,
                frets,
                position: *time,
                ..EofNote::empty()
            };

            eof_notes.push(eof_note);
            fingerings.push(chord.fingering.clone());
            tech_notes_list.push(vec![]);
        } else {
            // Determine crazy flag
            let crazy_flag = chord_opt.as_ref().and_then(|c| {
                let prev = if index > 0 { note_groups.get(index - 1) } else { None };
                let prev_chord = prev.and_then(|x| x.chord.as_ref());
                match prev_chord {
                    Some(prev_data) => {
                        let crazy = prev_data.chord_id == c.chord_id
                            && prev_data.handshape_id != c.handshape_id;
                        Some(crazy)
                    }
                    None => {
                        Some(c.is_full_panel && !c.is_first_in_hand_shape)
                    }
                }
            }).unwrap_or(false);
            let crazy_flag = if crazy_flag { EofNoteFlag::CRAZY } else { EofNoteFlag::empty() };

            // Find handshape template at this time
            let handshape_template: Option<&ChordTemplate> = inst.levels.get(*diff as usize)
                .and_then(|level| {
                    level.hand_shapes.iter()
                        .find(|hs| *time as i32 >= hs.start_time && (*time as i32) < hs.end_time)
                })
                .map(|hs| &inst.chord_templates[hs.chord_id as usize]);

            let bit_flag_from_hs: Option<u8> = handshape_template.map(bit_flag_from_template);

            let bit_flags: Vec<u8> = notes.iter().map(|n| get_bit_flag(n.string)).collect();
            let true_bit_flag = bit_flags.iter().fold(0u8, |acc, &b| acc | b);
            let common_bit_flag = bit_flag_from_hs.unwrap_or(true_bit_flag);
            let ghost_bit_flag = bit_flag_from_hs.map(|tbf| true_bit_flag ^ tbf).unwrap_or(0u8);

            let extended_note_flags: Vec<EofExtendedNoteFlag> = notes.iter()
                .map(|n| get_extended_note_flags(chord_opt.as_ref(), n))
                .collect();
            let common_ext_flags = if extended_note_flags.is_empty() {
                EofExtendedNoteFlag::empty()
            } else {
                extended_note_flags.iter().copied().reduce(|a, b| a & b).unwrap_or(EofExtendedNoteFlag::empty())
            };

            let split_flag = if chord_opt.is_none() && notes.len() > 1 {
                EofNoteFlag::SPLIT
            } else {
                EofNoteFlag::empty()
            };

            // Slide: all notes have same relative distance to slide target
            let slide: Option<u8> = {
                let slide_to = notes[0].slide_to;
                let distance = notes[0].fret - slide_to;
                if slide_to > 0 && notes.iter().all(|n| n.slide_to > 0 && n.fret - n.slide_to == distance) {
                    Some(slide_to as u8)
                } else {
                    None
                }
            };

            let unpitched_slide: Option<u8> = {
                let u_slide_to = notes[0].slide_unpitch_to;
                let distance = notes[0].fret - u_slide_to;
                if u_slide_to > 0 && notes.iter().all(|n| n.slide_unpitch_to > 0 && n.fret - n.slide_unpitch_to == distance) {
                    Some(u_slide_to as u8)
                } else {
                    None
                }
            };

            let note_flags: Vec<EofNoteFlag> = notes.iter().enumerate()
                .map(|(i, n)| get_note_flags(extended_note_flags[i], n))
                .collect();

            let mut common_flags = note_flags.iter().copied().reduce(|a, b| a & b).unwrap_or(EofNoteFlag::empty());
            if slide.is_none() {
                common_flags &= !(EofNoteFlag::SLIDE_DOWN | EofNoteFlag::SLIDE_UP | EofNoteFlag::RS_NOTATION);
            }
            if unpitched_slide.is_none() {
                common_flags &= !EofNoteFlag::UNPITCH_SLIDE;
            }

            let frets: Vec<u8> = if let Some(hs_tmpl) = handshape_template {
                frets_from_template(notes, hs_tmpl)
            } else {
                notes.iter().map(|n| {
                    if n.is_fret_hand_mute() { 128u8 | n.fret as u8 } else { n.fret as u8 }
                }).collect()
            };

            let max_sus = notes.iter().map(|n| n.sustain).max().unwrap_or(0);

            // Stop tech notes for split chords with different sustains
            let stop_tech_notes: Vec<EofNote> = if split_flag.is_empty() || notes.len() == 1 {
                vec![]
            } else {
                notes.iter()
                    .filter_map(|n| {
                        if max_sus - n.sustain > 3 {
                            Some(EofNote {
                                difficulty: *diff,
                                bit_flag: get_bit_flag(n.string),
                                position: (n.time + n.sustain) as u32,
                                flags: EofNoteFlag::EXTENDED_FLAGS,
                                extended_note_flags: EofExtendedNoteFlag::STOP,
                                actual_note_position: *time,
                                end_position: *time + max_sus as u32,
                                ..EofNote::empty()
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Per-string tech notes for divergent flags
            let tech_notes_flags: Vec<EofNote> = note_flags.iter().enumerate()
                .filter_map(|(i, &flag)| {
                    let ext = extended_note_flags[i];
                    if (flag & common_flags == flag) && (ext & common_ext_flags == ext) {
                        None
                    } else {
                        let n = &notes[i];
                        Some(EofNote {
                            difficulty: *diff,
                            bit_flag: bit_flags[i],
                            position: *time,
                            flags: flag & !common_flags,
                            slide_end_fret: if slide.is_none() && n.is_slide() {
                                Some(n.slide_to as u8)
                            } else {
                                None
                            },
                            unpitched_slide_end_fret: if unpitched_slide.is_none() && n.is_unpitched_slide() {
                                Some(n.slide_unpitch_to as u8)
                            } else {
                                None
                            },
                            extended_note_flags: ext & !common_ext_flags,
                            actual_note_position: *time,
                            end_position: *time + max_sus as u32,
                            ..EofNote::empty()
                        })
                    }
                })
                .collect();

            // Bend tech notes
            let bend_tech_notes: Vec<EofNote> = notes.iter()
                .filter(|n| n.is_bend())
                .flat_map(|n| {
                    let end_position = *time + max_sus as u32;
                    n.bend_values.iter()
                        .filter(|bv| !(bv.time == n.time && bv.step == 0.0))
                        .map(move |bv| {
                            let position = (bv.time as u32).max(*time);
                            EofNote {
                                difficulty: *diff,
                                bit_flag: get_bit_flag(n.string),
                                position,
                                flags: EofNoteFlag::RS_NOTATION | EofNoteFlag::BEND,
                                bend_strength: Some(convert_bend_value(bv.step)),
                                actual_note_position: *time,
                                end_position,
                                ..EofNote::empty()
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            let chord_name = chord_opt.as_ref().map(|x| x.template.chord_name.clone()).unwrap_or_default();

            let eof_note = EofNote {
                difficulty: *diff,
                chord_name,
                bit_flag: common_bit_flag,
                ghost_bit_flag,
                frets,
                position: *time,
                length: (max_sus as u32).max(1),
                flags: common_flags | split_flag | crazy_flag,
                slide_end_fret: slide,
                unpitched_slide_end_fret: unpitched_slide,
                extended_note_flags: common_ext_flags,
                ..EofNote::empty()
            };

            let fingering = if let Some(hs_tmpl) = handshape_template {
                fingering_from_template(hs_tmpl)
            } else if let Some(chord) = chord_opt {
                chord.fingering.clone()
            } else {
                vec![0u8; eof_note.frets.len()]
            };

            debug_assert_eq!(
                fingering.len(),
                eof_note.frets.len(),
                "fingering/frets length mismatch at time={} diff={}",
                time,
                diff
            );

            let all_tech: Vec<EofNote> = tech_notes_flags.into_iter()
                .chain(bend_tech_notes)
                .chain(stop_tech_notes)
                .collect();

            eof_notes.push(eof_note);
            fingerings.push(fingering);
            tech_notes_list.push(all_tech);
        }
    }

    (eof_notes, fingerings, tech_notes_list)
}
