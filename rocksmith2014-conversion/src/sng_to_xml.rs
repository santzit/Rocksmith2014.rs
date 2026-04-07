use rocksmith2014_sng::{
    Anchor as SngAnchor, Beat as SngBeat, BendData32, BendValue as SngBendValue, Chord as SngChord,
    Event as SngEvent, FingerPrint as SngFingerPrint, Level as SngLevel, Note as SngNote,
    NoteMask as SngNoteMask, PhraseExtraInfo, PhraseIteration as SngPhraseIteration,
    Section as SngSection, Sng, Tone as SngTone,
};
use rocksmith2014_xml::{
    Anchor as XmlAnchor, ArrangementEvent, BendValue as XmlBendValue, ChordNote, ChordTemplate,
    Ebeat, HandShape, HeroLevel, InstrumentalArrangement, Level as XmlLevel, Note as XmlNote,
    NoteMask as XmlNoteMask, Phrase as XmlPhrase, PhraseIteration as XmlPhraseIteration,
    PhraseProperty, Section as XmlSection, ToneChange,
};

use crate::utils::{bytes_to_string, sec_to_ms};

/// Converts an SNG Beat to an XML Ebeat.
pub fn convert_beat(beat: &SngBeat) -> Ebeat {
    let is_measure = beat
        .mask
        .contains(rocksmith2014_sng::BeatMask::FIRST_BEAT_OF_MEASURE);
    Ebeat {
        time: sec_to_ms(beat.time),
        measure: if is_measure { beat.measure } else { -1 },
    }
}

/// Converts an SNG Phrase to an XML Phrase.
pub fn convert_phrase(phrase: &rocksmith2014_sng::Phrase) -> XmlPhrase {
    XmlPhrase {
        max_difficulty: phrase.max_difficulty as u8,
        name: bytes_to_string(&phrase.name),
        disparity: phrase.disparity,
        ignore: phrase.ignore,
        solo: phrase.solo,
    }
}

/// Converts an SNG Chord (template) to an XML ChordTemplate.
pub fn convert_chord_template(chord: &SngChord) -> ChordTemplate {
    let name = bytes_to_string(&chord.name);
    let display_name = if chord.mask.contains(rocksmith2014_sng::ChordMask::ARPEGGIO) {
        format!("{}-arp", name)
    } else if chord.mask.contains(rocksmith2014_sng::ChordMask::NOP) {
        format!("{}-nop", name)
    } else {
        name.clone()
    };
    ChordTemplate {
        chord_name: name,
        display_name,
        fingers: chord.fingers,
        frets: chord.frets,
    }
}

/// Converts an SNG BendValue to an XML BendValue.
pub fn convert_bend_value(bv: &SngBendValue) -> XmlBendValue {
    XmlBendValue {
        time: sec_to_ms(bv.time),
        step: bv.step as f64,
        unk5: 0,
    }
}

/// Converts SNG BendData32 to a Vec of XML BendValues (or empty if unused).
pub fn convert_bend_data32(bd: &BendData32) -> Vec<XmlBendValue> {
    let count = bd.used_count as usize;
    bd.bend_values[..count]
        .iter()
        .map(convert_bend_value)
        .collect()
}

/// Converts an SNG PhraseIteration to an XML PhraseIteration.
pub fn convert_phrase_iteration(pi: &SngPhraseIteration) -> XmlPhraseIteration {
    XmlPhraseIteration {
        time: sec_to_ms(pi.start_time),
        end_time: sec_to_ms(pi.end_time),
        phrase_id: pi.phrase_id as u32,
        hero_levels: Some(vec![
            HeroLevel {
                hero: 1,
                difficulty: pi.difficulty[0],
            },
            HeroLevel {
                hero: 2,
                difficulty: pi.difficulty[1],
            },
            HeroLevel {
                hero: 3,
                difficulty: pi.difficulty[2],
            },
        ]),
    }
}

/// Converts an SNG PhraseExtraInfo to an XML PhraseProperty.
pub fn convert_phrase_extra_info(info: &PhraseExtraInfo) -> PhraseProperty {
    PhraseProperty {
        phrase_id: info.phrase_id,
        redundant: info.redundant as i32,
        level_jump: info.level_jump as i32,
        empty: info.empty,
        difficulty: info.difficulty,
    }
}

/// Converts an SNG Event to an XML ArrangementEvent.
pub fn convert_event(event: &SngEvent) -> ArrangementEvent {
    ArrangementEvent {
        time: sec_to_ms(event.time),
        code: bytes_to_string(&event.name),
    }
}

/// Converts an SNG Tone to an XML ToneChange.
/// `tone_names` should index by tone_id (0-3 → Tone_A..D).
pub fn convert_tone(tone: &SngTone, tone_names: &[&str]) -> ToneChange {
    let name = tone_names
        .get(tone.tone_id as usize)
        .copied()
        .unwrap_or("")
        .to_string();
    ToneChange {
        time: sec_to_ms(tone.time),
        name,
        id: tone.tone_id,
    }
}

/// Converts an SNG Section to an XML Section.
pub fn convert_section(section: &SngSection) -> XmlSection {
    XmlSection {
        name: bytes_to_string(&section.name),
        number: section.number,
        start_time: sec_to_ms(section.start_time),
        end_time: sec_to_ms(section.end_time),
    }
}

/// Converts an SNG Anchor to an XML Anchor.
pub fn convert_anchor(anchor: &SngAnchor) -> XmlAnchor {
    XmlAnchor {
        time: sec_to_ms(anchor.start_time),
        end_time: sec_to_ms(anchor.end_time),
        fret: anchor.fret_id,
        width: anchor.width,
    }
}

/// Converts an SNG FingerPrint to an XML HandShape.
pub fn convert_hand_shape(fp: &SngFingerPrint) -> HandShape {
    HandShape {
        chord_id: fp.chord_id,
        start_time: sec_to_ms(fp.start_time),
        end_time: sec_to_ms(fp.end_time),
    }
}

/// Converts the SNG NoteMask into an XML NoteMask.
fn sng_note_mask_to_xml(sng_mask: SngNoteMask) -> XmlNoteMask {
    let mut mask = XmlNoteMask::empty();
    if sng_mask.contains(SngNoteMask::HAMMER_ON) {
        mask |= XmlNoteMask::HAMMER_ON;
    }
    if sng_mask.contains(SngNoteMask::PULL_OFF) {
        mask |= XmlNoteMask::PULL_OFF;
    }
    if sng_mask.contains(SngNoteMask::ACCENT) {
        mask |= XmlNoteMask::ACCENT;
    }
    if sng_mask.contains(SngNoteMask::MUTE) {
        mask |= XmlNoteMask::FRET_HAND_MUTE;
    }
    if sng_mask.contains(SngNoteMask::HARMONIC) {
        mask |= XmlNoteMask::HARMONIC;
    }
    if sng_mask.contains(SngNoteMask::IGNORE) {
        mask |= XmlNoteMask::IGNORE;
    }
    if sng_mask.contains(SngNoteMask::PALM_MUTE) {
        mask |= XmlNoteMask::PALM_MUTE;
    }
    if sng_mask.contains(SngNoteMask::PINCH_HARMONIC) {
        mask |= XmlNoteMask::PINCH_HARMONIC;
    }
    if sng_mask.contains(SngNoteMask::TREMOLO) {
        mask |= XmlNoteMask::TREMOLO;
    }
    if sng_mask.contains(SngNoteMask::PARENT) {
        mask |= XmlNoteMask::LINK_NEXT;
    }
    if sng_mask.contains(SngNoteMask::RIGHT_HAND) {
        mask |= XmlNoteMask::RIGHT_HAND;
    }
    mask
}

/// Converts an SNG Note (single note, not chord) into an XML Note.
pub fn convert_note(note: &SngNote) -> XmlNote {
    let sng_mask = note.mask;
    let mut mask = sng_note_mask_to_xml(sng_mask);

    let slap = if sng_mask.contains(SngNoteMask::SLAP) {
        mask |= XmlNoteMask::SLAP;
        note.slap
    } else {
        -1
    };
    let pluck = if sng_mask.contains(SngNoteMask::PLUCK) {
        mask |= XmlNoteMask::PLUCK;
        note.pluck
    } else {
        -1
    };

    XmlNote {
        time: sec_to_ms(note.time),
        string: note.string_index,
        fret: note.fret,
        sustain: sec_to_ms(note.sustain),
        vibrato: note.vibrato as i8,
        slide_to: note.slide_to,
        slide_unpitch_to: note.slide_unpitch_to,
        left_hand: note.left_hand,
        tap: note.tap.max(0),
        pick_direction: if note.pick_direction > 0 { 1 } else { 0 },
        slap,
        pluck,
        max_bend: note.max_bend as f64,
        mask,
        bend_values: note.bend_data.iter().map(convert_bend_value).collect(),
    }
}

/// Creates XML ChordNotes for a chord, reading from SNG chord_notes data.
fn create_xml_chord_notes(sng: &Sng, chord_note: &SngNote) -> Vec<ChordNote> {
    let chord_id = chord_note.chord_id as usize;
    if chord_id >= sng.chords.len() {
        return vec![];
    }
    let template = &sng.chords[chord_id];
    let cn_data = if chord_note.chord_notes_id >= 0 {
        let cn_id = chord_note.chord_notes_id as usize;
        if cn_id < sng.chord_notes.len() {
            Some(&sng.chord_notes[cn_id])
        } else {
            None
        }
    } else {
        None
    };

    let mut result = Vec::new();
    for i in 0..6usize {
        if template.frets[i] == -1 {
            continue;
        }
        let (cn_mask, slide_to, slide_unpitch_to, vibrato, bend_values) = if let Some(cn) = cn_data
        {
            let m = rocksmith2014_sng::NoteMask::from_bits_truncate(cn.mask[i]);
            let bv = if cn.bend_data[i].used_count > 0 {
                convert_bend_data32(&cn.bend_data[i])
            } else {
                vec![]
            };
            (
                sng_note_mask_to_xml(m),
                cn.slide_to[i],
                cn.slide_unpitch_to[i],
                cn.vibrato[i] as i8,
                bv,
            )
        } else {
            (XmlNoteMask::empty(), -1, -1, 0, vec![])
        };

        result.push(ChordNote {
            string: i as i8,
            fret: template.frets[i],
            sustain: sec_to_ms(chord_note.sustain),
            vibrato,
            slide_to,
            slide_unpitch_to,
            left_hand: template.fingers[i],
            bend_values,
            mask: cn_mask,
        });
    }
    result
}

/// Converts an SNG chord Note into an XML Chord.
pub fn convert_chord(sng: &Sng, chord: &SngNote) -> rocksmith2014_xml::Chord {
    let has_chord_panel = chord.mask.contains(SngNoteMask::CHORD_PANEL);
    let chord_notes = if has_chord_panel {
        create_xml_chord_notes(sng, chord)
    } else {
        vec![]
    };

    let mut xml_mask = rocksmith2014_xml::ChordMask::empty();
    let sng_mask = chord.mask;
    if sng_mask.contains(SngNoteMask::PARENT) {
        xml_mask |= rocksmith2014_xml::ChordMask::LINK_NEXT;
    }
    if sng_mask.contains(SngNoteMask::ACCENT) {
        xml_mask |= rocksmith2014_xml::ChordMask::ACCENT;
    }
    if sng_mask.contains(SngNoteMask::FRET_HAND_MUTE) {
        xml_mask |= rocksmith2014_xml::ChordMask::FRET_HAND_MUTE;
    }
    if sng_mask.contains(SngNoteMask::HIGH_DENSITY) {
        xml_mask |= rocksmith2014_xml::ChordMask::HIGH_DENSITY;
    }
    if sng_mask.contains(SngNoteMask::IGNORE) {
        xml_mask |= rocksmith2014_xml::ChordMask::IGNORE;
    }
    if sng_mask.contains(SngNoteMask::PALM_MUTE) {
        xml_mask |= rocksmith2014_xml::ChordMask::PALM_MUTE;
    }

    rocksmith2014_xml::Chord {
        time: sec_to_ms(chord.time),
        chord_id: chord.chord_id,
        sustain: sec_to_ms(chord.sustain),
        mask: xml_mask,
        chord_notes,
    }
}

/// Converts an SNG Level to an XML Level.
pub fn convert_level(sng: &Sng, level: &SngLevel) -> XmlLevel {
    let mut notes = Vec::new();
    let mut chords = Vec::new();

    for sng_note in &level.notes {
        if sng_note.chord_id == -1 {
            notes.push(convert_note(sng_note));
        } else {
            chords.push(convert_chord(sng, sng_note));
        }
    }

    // Sort notes and chords by time
    notes.sort_by_key(|n| n.time);
    chords.sort_by_key(|c| c.time);

    let anchors = level.anchors.iter().map(convert_anchor).collect();
    let mut hand_shapes: Vec<HandShape> = level
        .hand_shapes
        .iter()
        .chain(level.arpeggios.iter())
        .map(convert_hand_shape)
        .collect();
    hand_shapes.sort_by_key(|hs| hs.start_time);

    XmlLevel {
        difficulty: level.difficulty as i8,
        anchors,
        hand_shapes,
        notes,
        chords,
    }
}

/// Converts a full SNG arrangement into an XML InstrumentalArrangement.
#[allow(clippy::field_reassign_with_default)]
pub fn sng_to_xml(sng: &Sng) -> InstrumentalArrangement {
    let ebeats = sng.beats.iter().map(convert_beat).collect();
    let phrases = sng.phrases.iter().map(convert_phrase).collect();
    let phrase_iterations = sng
        .phrase_iterations
        .iter()
        .map(convert_phrase_iteration)
        .collect();
    let phrase_properties = sng
        .phrase_extra_info
        .iter()
        .map(convert_phrase_extra_info)
        .collect();
    let chord_templates = sng.chords.iter().map(convert_chord_template).collect();
    let events = sng.events.iter().map(convert_event).collect();
    let sections = sng.sections.iter().map(convert_section).collect();
    let levels = sng.levels.iter().map(|l| convert_level(sng, l)).collect();
    let tones = sng.tones.iter().map(|t| convert_tone(t, &[])).collect();

    let meta = {
        let mut m = rocksmith2014_xml::MetaData::default();
        m.song_length = sec_to_ms(sng.metadata.song_length);
        m.last_conversion_date_time = bytes_to_string(&sng.metadata.last_conversion_date_time);
        m.part = sng.metadata.part as i32;
        m.capo = sng.metadata.capo_fret_id.max(0);
        for (i, &t) in sng.metadata.tuning.iter().enumerate().take(6) {
            m.tuning.strings[i] = t;
        }
        m
    };

    InstrumentalArrangement {
        meta,
        ebeats,
        phrases,
        phrase_iterations,
        phrase_properties,
        chord_templates,
        events,
        sections,
        levels,
        tones,
        ..Default::default()
    }
}

/// Returns the raw bend data as Option<Vec<XmlBendValue>> for test compatibility.
#[allow(dead_code)]
pub fn convert_bend_data32_opt(bd: &BendData32) -> Option<Vec<XmlBendValue>> {
    if bd.used_count == 0 {
        None
    } else {
        Some(convert_bend_data32(bd))
    }
}
