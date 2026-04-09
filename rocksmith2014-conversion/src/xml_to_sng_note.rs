use rocksmith2014_sng::{
    AnchorExtension, BendValue as SngBendValue, ChordNotes, FingerPrint, Note as SngNote,
    NoteMask as SngNoteMask,
};
use rocksmith2014_xml::{
    ChordMask as XmlChordMask, InstrumentalArrangement, Note as XmlNote, NoteMask as XmlNoteMask,
};

use crate::{
    accu_data::AccuData,
    utils::{find_anchor, find_phrase_iteration_id, ms_to_sec},
    xml_to_sng::{convert_bend_value, XmlEntity},
};

/// Flags for note marking (affects `note.flags` field).
pub type FlagFn = fn(Option<&SngNote>, &SngNote) -> u32;

/// Never flags notes (flags = 0).
pub fn flag_never(_prev: Option<&SngNote>, _curr: &SngNote) -> u32 {
    0
}

/// Flags a note when the anchor changes (used for MIDI display).
pub fn flag_on_anchor_change(prev: Option<&SngNote>, curr: &SngNote) -> u32 {
    if curr.fret == 0 {
        return 0;
    }
    match prev {
        Some(p) if p.anchor_fret != curr.anchor_fret => 1,
        Some(_) => 0,
        None => 1,
    }
}

/// State for the stateful note converter.
#[allow(dead_code)]
pub struct NoteConverter<'a> {
    note_times: &'a [i32],
    pi_times: &'a [i32],
    hand_shapes: &'a [FingerPrint],
    arpeggios: &'a [FingerPrint],
    pub accu_data: &'a mut AccuData,
    flag_fn: FlagFn,
    arr: &'a InstrumentalArrangement,
    difficulty: usize,
    // Mutable conversion state
    pending_link_nexts: [(Option<XmlNote>, i16); 6],
    previous_note: Option<SngNote>,
}

impl<'a> NoteConverter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        note_times: &'a [i32],
        pi_times: &'a [i32],
        hand_shapes: &'a [FingerPrint],
        arpeggios: &'a [FingerPrint],
        accu_data: &'a mut AccuData,
        flag_fn: FlagFn,
        arr: &'a InstrumentalArrangement,
        difficulty: usize,
    ) -> Self {
        let pending_link_nexts: [(Option<XmlNote>, i16); 6] =
            std::array::from_fn(|_| (None, -1i16));
        Self {
            note_times,
            pi_times,
            hand_shapes,
            arpeggios,
            accu_data,
            flag_fn,
            arr,
            difficulty,
            pending_link_nexts,
            previous_note: None,
        }
    }

    /// Converts an XML entity (note or chord) to an SNG Note.
    pub fn call(&mut self, index: usize, entity: XmlEntity) -> SngNote {
        match entity {
            XmlEntity::Note(note) => self.convert_single_note(index, &note),
            XmlEntity::Chord(chord) => self.convert_chord_note(index, &chord),
        }
    }

    fn convert_single_note(&mut self, index: usize, note: &XmlNote) -> SngNote {
        let time = note.time;
        let stime = ms_to_sec(time);

        let pi_id = find_phrase_iteration_id(time, &self.arr.phrase_iterations);
        let phrase_id = self
            .arr
            .phrase_iterations
            .get(pi_id)
            .map(|pi| pi.phrase_id)
            .unwrap_or(0) as i32;

        let anchor = find_anchor(time, &self.arr.levels[self.difficulty].anchors);
        let anchor_fret = anchor.fret;
        let anchor_width = anchor.width as i8;

        // Hand shape IDs
        let hs_id = crate::utils::find_finger_print_id(stime, self.hand_shapes);
        let arp_id = crate::utils::find_finger_print_id(stime, self.arpeggios);

        // Build SNG mask
        let xml_mask = note.mask;
        let mut sng_mask = SngNoteMask::SINGLE;

        if note.fret == 0 {
            sng_mask |= SngNoteMask::OPEN;
        }
        if note.sustain > 0 {
            sng_mask |= SngNoteMask::SUSTAIN;
        }
        if xml_mask.contains(XmlNoteMask::ACCENT) {
            sng_mask |= SngNoteMask::ACCENT;
        }
        if xml_mask.contains(XmlNoteMask::HAMMER_ON) {
            sng_mask |= SngNoteMask::HAMMER_ON;
        }
        if xml_mask.contains(XmlNoteMask::HARMONIC) {
            sng_mask |= SngNoteMask::HARMONIC;
        }
        if xml_mask.contains(XmlNoteMask::IGNORE) {
            sng_mask |= SngNoteMask::IGNORE;
        }
        if xml_mask.contains(XmlNoteMask::FRET_HAND_MUTE) {
            sng_mask |= SngNoteMask::MUTE;
        }
        if xml_mask.contains(XmlNoteMask::PALM_MUTE) {
            sng_mask |= SngNoteMask::PALM_MUTE;
        }
        if xml_mask.contains(XmlNoteMask::PULL_OFF) {
            sng_mask |= SngNoteMask::PULL_OFF;
        }
        if xml_mask.contains(XmlNoteMask::TREMOLO) {
            sng_mask |= SngNoteMask::TREMOLO;
        }
        if xml_mask.contains(XmlNoteMask::PINCH_HARMONIC) {
            sng_mask |= SngNoteMask::PINCH_HARMONIC;
        }
        if xml_mask.contains(XmlNoteMask::RIGHT_HAND) {
            sng_mask |= SngNoteMask::RIGHT_HAND;
        }
        if xml_mask.contains(XmlNoteMask::SLAP) || note.slap > 0 {
            sng_mask |= SngNoteMask::SLAP;
        }
        if xml_mask.contains(XmlNoteMask::PLUCK) || note.pluck > 0 {
            sng_mask |= SngNoteMask::PLUCK;
        }
        if note.slide_to >= 0 {
            sng_mask |= SngNoteMask::SLIDE;
        }
        if note.slide_unpitch_to >= 0 {
            sng_mask |= SngNoteMask::UNPITCHED_SLIDE;
        }
        if note.tap > 0 {
            sng_mask |= SngNoteMask::TAP;
        }
        if note.vibrato > 0 {
            sng_mask |= SngNoteMask::VIBRATO;
        }
        if !note.bend_values.is_empty() {
            sng_mask |= SngNoteMask::BEND;
        }
        if note.left_hand >= 0 {
            sng_mask |= SngNoteMask::LEFT_HAND;
        }
        if arp_id >= 0 {
            sng_mask |= SngNoteMask::ARPEGGIO;
        }

        // Link-next / parent-child
        let is_link_next = xml_mask.contains(XmlNoteMask::LINK_NEXT);
        if is_link_next {
            sng_mask |= SngNoteMask::PARENT;
        }

        // Check if this note is a child of a pending link-next
        let string_idx = note.string as usize;
        let parent_prev_note =
            if let (Some(_pending), parent_idx) = &self.pending_link_nexts[string_idx] {
                sng_mask |= SngNoteMask::CHILD;
                *parent_idx
            } else {
                -1i16
            };

        // Next/prev note indices
        let prev_idx = if index > 0 { index as i16 - 1 } else { -1i16 };
        let next_idx = if index + 1 < self.note_times.len() {
            index as i16 + 1
        } else {
            -1i16
        };

        // Determine max bend
        let max_bend = note
            .bend_values
            .iter()
            .map(|bv| bv.step as f32)
            .fold(0.0f32, f32::max);

        let bend_data: Vec<SngBendValue> =
            note.bend_values.iter().map(convert_bend_value).collect();

        let slap = if sng_mask.contains(SngNoteMask::SLAP) {
            1i8
        } else {
            -1i8
        };
        let pluck = if sng_mask.contains(SngNoteMask::PLUCK) {
            1i8
        } else {
            -1i8
        };

        let mut sng_note = SngNote {
            mask: sng_mask,
            flags: 0,
            hash: 0,
            time: stime,
            string_index: note.string,
            fret: note.fret,
            anchor_fret,
            anchor_width,
            chord_id: -1,
            chord_notes_id: -1,
            phrase_id,
            phrase_iteration_id: pi_id as i32,
            finger_print_id: [hs_id, arp_id],
            next_iter_note: next_idx,
            prev_iter_note: prev_idx,
            parent_prev_note,
            slide_to: note.slide_to,
            slide_unpitch_to: note.slide_unpitch_to,
            left_hand: note.left_hand,
            tap: note.tap,
            pick_direction: note.pick_direction,
            slap,
            pluck,
            vibrato: note.vibrato as i16,
            sustain: ms_to_sec(note.sustain),
            max_bend,
            bend_data,
        };

        // Apply flag function
        sng_note.flags = (self.flag_fn)(self.previous_note.as_ref(), &sng_note);

        // Create anchor extension for slides
        if note.slide_to >= 0 && note.sustain > 0 {
            let ext_time = stime + ms_to_sec(note.sustain);
            if self.difficulty < self.accu_data.anchor_extensions.len() {
                self.accu_data.anchor_extensions[self.difficulty].push(AnchorExtension {
                    beat_time: ext_time,
                    fret_id: note.slide_to,
                });
            }
        }

        // Track note counts
        let is_ignored = sng_mask.contains(SngNoteMask::IGNORE);
        if let Some(pi) = self.arr.phrase_iterations.get(pi_id) {
            self.accu_data
                .add_note(pi_id, self.difficulty, pi, is_ignored);
        }

        // Track string mask
        let section_id = crate::utils::find_section_id(time, &self.arr.sections);
        self.accu_data
            .update_string_mask(section_id, self.difficulty, note.string as usize);

        // Update pending link-next
        if is_link_next {
            self.pending_link_nexts[string_idx] = (Some(note.clone()), index as i16);
        } else {
            self.pending_link_nexts[string_idx] = (None, -1);
        }

        self.previous_note = Some(sng_note.clone());
        sng_note
    }

    fn convert_chord_note(&mut self, index: usize, chord: &rocksmith2014_xml::Chord) -> SngNote {
        let time = chord.time;
        let stime = ms_to_sec(time);

        let pi_id = find_phrase_iteration_id(time, &self.arr.phrase_iterations);
        let phrase_id = self
            .arr
            .phrase_iterations
            .get(pi_id)
            .map(|pi| pi.phrase_id)
            .unwrap_or(0) as i32;

        let anchor = find_anchor(time, &self.arr.levels[self.difficulty].anchors);
        let anchor_fret = anchor.fret;
        let anchor_width = anchor.width as i8;

        let xml_chord_mask = chord.mask;

        // Determine chord template
        let chord_id = chord.chord_id;
        let template = self.arr.chord_templates.get(chord_id as usize);

        // Check if double-stop (only 2 strings played)
        let is_double_stop = template
            .map(|t| t.frets.iter().filter(|&&f| f != -1).count() == 2)
            .unwrap_or(false);

        // Check arpeggio (display name ends with -arp)
        let is_arpeggio = template
            .map(|t| t.display_name.ends_with("-arp"))
            .unwrap_or(false);

        let hs_id = crate::utils::find_finger_print_id(stime, self.hand_shapes);
        let arp_id = crate::utils::find_finger_print_id(stime, self.arpeggios);

        // Build mask
        let mut sng_mask = SngNoteMask::CHORD;

        if xml_chord_mask.contains(XmlChordMask::ACCENT) {
            sng_mask |= SngNoteMask::ACCENT;
        }
        if xml_chord_mask.contains(XmlChordMask::FRET_HAND_MUTE) {
            sng_mask |= SngNoteMask::FRET_HAND_MUTE;
        }
        if xml_chord_mask.contains(XmlChordMask::HIGH_DENSITY) {
            sng_mask |= SngNoteMask::HIGH_DENSITY;
        }
        if xml_chord_mask.contains(XmlChordMask::IGNORE) {
            sng_mask |= SngNoteMask::IGNORE;
        }
        if xml_chord_mask.contains(XmlChordMask::PALM_MUTE) {
            sng_mask |= SngNoteMask::PALM_MUTE;
        }
        if is_double_stop {
            sng_mask |= SngNoteMask::DOUBLE_STOP;
        }
        if is_arpeggio || arp_id >= 0 {
            sng_mask |= SngNoteMask::ARPEGGIO;
        }

        // Link-next / parent-child
        let is_link_next = xml_chord_mask.contains(XmlChordMask::LINK_NEXT);
        if is_link_next {
            sng_mask |= SngNoteMask::PARENT;
        }

        // Check if child of pending link-next chord
        // For chords, check if any chord note is a child
        let parent_prev_note = {
            let mut found = -1i16;
            for cn in &chord.chord_notes {
                let s = cn.string as usize;
                if s < 6 {
                    if let (Some(_), parent_idx) = &self.pending_link_nexts[s] {
                        found = *parent_idx;
                        break;
                    }
                }
            }
            found
        };
        if parent_prev_note >= 0 {
            sng_mask |= SngNoteMask::CHILD;
        }

        // Has chord notes that need SNG chord notes?
        let has_chord_panel = !chord.chord_notes.is_empty() && !is_double_stop && !is_arpeggio;
        if has_chord_panel {
            sng_mask |= SngNoteMask::CHORD_PANEL;
        }

        // Create SNG chord notes if needed (with deduplication like .NET ChordNotesMap)
        let (sng_chord_notes_id, _needs_chord_notes) =
            if !chord.chord_notes.is_empty() && should_create_chord_notes(&chord.chord_notes) {
                let cn = build_sng_chord_notes(&chord.chord_notes, &[]);
                // Serialize to bytes for dedup (f32 compared by bits)
                let key = chord_notes_key(&cn);
                // Look for existing identical entry
                let id = if let Some(pos) = self
                    .accu_data
                    .chord_notes_keys
                    .iter()
                    .position(|k| *k == key)
                {
                    pos as i32
                } else {
                    let id = self.accu_data.chord_notes.len() as i32;
                    self.accu_data.chord_notes_keys.push(key);
                    self.accu_data.chord_notes.push(cn);
                    id
                };
                sng_mask |= SngNoteMask::CHORD_NOTES;
                (id, true)
            } else {
                (-1, false)
            };

        // Sustain from chord notes
        let sustain = chord
            .chord_notes
            .first()
            .map(|cn| ms_to_sec(cn.sustain))
            .unwrap_or(0.0);

        if sustain > 0.0 {
            sng_mask |= SngNoteMask::SUSTAIN;
        }

        let prev_idx = if index > 0 { index as i16 - 1 } else { -1i16 };
        let next_idx = if index + 1 < self.note_times.len() {
            index as i16 + 1
        } else {
            -1i16
        };

        let mut sng_note = SngNote {
            mask: sng_mask,
            flags: 0,
            hash: 0,
            time: stime,
            string_index: -1,
            fret: -1,
            anchor_fret,
            anchor_width,
            chord_id,
            chord_notes_id: sng_chord_notes_id,
            phrase_id,
            phrase_iteration_id: pi_id as i32,
            finger_print_id: [hs_id, arp_id],
            next_iter_note: next_idx,
            prev_iter_note: prev_idx,
            parent_prev_note,
            slide_to: -1,
            slide_unpitch_to: -1,
            left_hand: -1,
            tap: -1,
            pick_direction: -1,
            slap: -1,
            pluck: -1,
            vibrato: 0,
            sustain,
            max_bend: 0.0,
            bend_data: vec![],
        };

        sng_note.flags = (self.flag_fn)(self.previous_note.as_ref(), &sng_note);

        // Track note counts
        let is_ignored = sng_mask.contains(SngNoteMask::IGNORE);
        if let Some(pi) = self.arr.phrase_iterations.get(pi_id) {
            self.accu_data
                .add_note(pi_id, self.difficulty, pi, is_ignored);
        }

        // Track string mask for each played string
        let section_id = crate::utils::find_section_id(time, &self.arr.sections);
        for cn in &chord.chord_notes {
            self.accu_data
                .update_string_mask(section_id, self.difficulty, cn.string as usize);
        }

        // Update pending link-next for chord notes that have link-next
        for cn in &chord.chord_notes {
            let s = cn.string as usize;
            if s < 6 {
                if is_link_next && cn.mask.contains(XmlNoteMask::LINK_NEXT) {
                    // Create a synthetic note to act as pending parent for this string
                    let synthetic = XmlNote {
                        time,
                        string: s as i8,
                        fret: cn.fret,
                        ..Default::default()
                    };
                    self.pending_link_nexts[s] = (Some(synthetic), index as i16);
                } else {
                    self.pending_link_nexts[s] = (None, -1);
                }
            }
        }

        self.previous_note = Some(sng_note.clone());
        sng_note
    }
}

/// Returns true if any chord note would produce a non-None SNG mask,
/// following the .NET `createMaskForChordNote` logic.
fn should_create_chord_notes(chord_notes: &[rocksmith2014_xml::ChordNote]) -> bool {
    chord_notes.iter().any(|cn| {
        cn.fret == 0  // OPEN
            || cn.sustain > 0  // SUSTAIN
            || cn.slide_to >= 0
            || cn.slide_unpitch_to >= 0
            || cn.vibrato != 0
            || !cn.bend_values.is_empty()
            || cn.mask.intersects(
                XmlNoteMask::LINK_NEXT
                    | XmlNoteMask::ACCENT
                    | XmlNoteMask::TREMOLO
                    | XmlNoteMask::FRET_HAND_MUTE
                    | XmlNoteMask::HAMMER_ON
                    | XmlNoteMask::HARMONIC
                    | XmlNoteMask::PALM_MUTE
                    | XmlNoteMask::PINCH_HARMONIC,
            )
    })
}

/// Builds an SNG ChordNotes from XML chord notes.
fn build_sng_chord_notes(
    chord_notes: &[rocksmith2014_xml::ChordNote],
    _extra: &[()],
) -> ChordNotes {
    let mut result = ChordNotes::default();

    for cn in chord_notes {
        let s = cn.string as usize;
        if s >= 6 {
            continue;
        }

        // Set mask following .NET createMaskForChordNote logic
        let cn_xml_mask = cn.mask;
        let mut sng_m = 0u32;
        if cn.fret == 0 {
            sng_m |= SngNoteMask::OPEN.bits();
        }
        if cn.sustain > 0 {
            sng_m |= SngNoteMask::SUSTAIN.bits();
        }
        if cn.slide_to >= 0 {
            sng_m |= SngNoteMask::SLIDE.bits();
        }
        if cn.slide_unpitch_to >= 0 {
            sng_m |= SngNoteMask::UNPITCHED_SLIDE.bits();
        }
        if cn.vibrato != 0 {
            sng_m |= SngNoteMask::VIBRATO.bits();
        }
        if !cn.bend_values.is_empty() {
            sng_m |= SngNoteMask::BEND.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::LINK_NEXT) {
            sng_m |= SngNoteMask::PARENT.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::ACCENT) {
            sng_m |= SngNoteMask::ACCENT.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::TREMOLO) {
            sng_m |= SngNoteMask::TREMOLO.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::FRET_HAND_MUTE) {
            sng_m |= SngNoteMask::MUTE.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::HAMMER_ON) {
            sng_m |= SngNoteMask::HAMMER_ON.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::HARMONIC) {
            sng_m |= SngNoteMask::HARMONIC.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::PALM_MUTE) {
            sng_m |= SngNoteMask::PALM_MUTE.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::PINCH_HARMONIC) {
            sng_m |= SngNoteMask::PINCH_HARMONIC.bits();
        }
        if cn_xml_mask.contains(XmlNoteMask::PULL_OFF) {
            sng_m |= SngNoteMask::PULL_OFF.bits();
        }
        result.mask[s] = sng_m;

        result.slide_to[s] = if cn.slide_to >= 0 { cn.slide_to } else { -1 };
        result.slide_unpitch_to[s] = if cn.slide_unpitch_to >= 0 {
            cn.slide_unpitch_to
        } else {
            -1
        };
        result.vibrato[s] = cn.vibrato as i16;

        // Bend data
        if !cn.bend_values.is_empty() {
            let count = cn.bend_values.len().min(32) as i32;
            result.bend_data[s].used_count = count;
            for (i, bv) in cn.bend_values.iter().enumerate().take(32) {
                result.bend_data[s].bend_values[i] = convert_bend_value(bv);
            }
        }
    }

    // Fill unset slide_to/slide_unpitch_to with -1
    for i in 0..6 {
        if result.slide_to[i] == 0 && result.mask[i] == 0 {
            // Check if string was mentioned in chord_notes
            if !chord_notes.iter().any(|cn| cn.string as usize == i) {
                result.slide_to[i] = -1;
                result.slide_unpitch_to[i] = -1;
            }
        }
    }

    result
}

/// Produces a byte-level key for a ChordNotes struct for deduplication.
/// Follows .NET's structural equality semantics for the ChordNotesMap dictionary.
fn chord_notes_key(cn: &ChordNotes) -> Vec<u8> {
    let mut key = Vec::with_capacity(6 * (4 + 1 + 1 + 2) + 6 * 32 * 12 + 6 * 4);
    for &m in &cn.mask {
        key.extend_from_slice(&m.to_le_bytes());
    }
    for bd in &cn.bend_data {
        for bv in &bd.bend_values {
            key.extend_from_slice(&bv.time.to_bits().to_le_bytes());
            key.extend_from_slice(&bv.step.to_bits().to_le_bytes());
            key.extend_from_slice(&bv.unused.to_le_bytes());
        }
        key.extend_from_slice(&bd.used_count.to_le_bytes());
    }
    for &v in &cn.slide_to {
        key.push(v as u8);
    }
    for &v in &cn.slide_unpitch_to {
        key.push(v as u8);
    }
    for &v in &cn.vibrato {
        key.extend_from_slice(&v.to_le_bytes());
    }
    key
}
