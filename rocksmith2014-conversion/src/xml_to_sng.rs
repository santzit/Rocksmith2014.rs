use rocksmith2014_sng::{
    Anchor as SngAnchor, AnchorExtension, Beat as SngBeat, BeatMask, BendValue as SngBendValue,
    Chord as SngChord, ChordMask as SngChordMask, ChordNotes, Event as SngEvent, FingerPrint,
    MetaData as SngMetaData, Note as SngNote, NoteMask as SngNoteMask, Phrase as SngPhrase,
    PhraseExtraInfo, PhraseIteration as SngPhraseIteration, Section as SngSection, Sng,
    Tone as SngTone, DNA,
};
use rocksmith2014_xml::{
    Anchor as XmlAnchor, ArrangementEvent, BendValue as XmlBendValue, ChordMask as XmlChordMask,
    ChordTemplate, Ebeat, HandShape, InstrumentalArrangement, Level as XmlLevel, Note as XmlNote,
    NoteMask as XmlNoteMask, Phrase as XmlPhrase, PhraseIteration as XmlPhraseIteration,
    PhraseProperty, Section as XmlSection, ToneChange,
};

use crate::utils::{
    bytes_to_string, find_beat_phrase_iteration_id, find_phrase_iteration_id, ms_to_sec, sec_to_ms,
    string_to_bytes,
};

/// Midi base notes for standard tuning (strings 0-5: E2 A2 D3 G3 B3 E4).
const STANDARD_TUNING_MIDI: [i32; 6] = [40, 45, 50, 55, 59, 64];

/// Computes a MIDI note value for a given string/fret with tuning offsets.
pub fn to_midi_note(string: usize, fret: i8, tuning: &[i16; 6], capo: i8, is_bass: bool) -> i32 {
    let fret = if capo > 0 && fret == 0 {
        capo as i32
    } else {
        fret as i32
    };
    let offset = if is_bass { -12 } else { 0 };
    STANDARD_TUNING_MIDI[string] + tuning[string] as i32 + fret + offset
}

/// Maps frets to MIDI notes.
pub fn map_to_midi_notes(frets: &[i8; 6], tuning: &[i16; 6], capo: i8, is_bass: bool) -> [i32; 6] {
    std::array::from_fn(|s| {
        if frets[s] == -1 {
            -1
        } else {
            to_midi_note(s, frets[s], tuning, capo, is_bass)
        }
    })
}

/// Creates an array of phrase iteration start times (with song length as last element).
pub fn create_phrase_iteration_times_array(arr: &InstrumentalArrangement) -> Vec<i32> {
    let n = arr.phrase_iterations.len();
    let mut times = Vec::with_capacity(n + 1);
    for pi in &arr.phrase_iterations {
        times.push(pi.time);
    }
    times.push(arr.meta.song_length);
    times
}

/// Converts an XML Ebeat to an SNG Beat.
/// Returns a closure (stateful) that tracks the current measure number and beat counter.
pub fn make_beat_converter(arr: &InstrumentalArrangement) -> impl FnMut(&Ebeat) -> SngBeat + '_ {
    let pi_count = arr.phrase_iterations.len();
    let phrase_iterations = &arr.phrase_iterations;

    let mut current_measure: i16 = -1;
    let mut current_beat: i16 = 0;
    let mut is_even = false;

    move |ebeat: &Ebeat| -> SngBeat {
        let is_first = ebeat.measure >= 0;

        if is_first {
            current_measure = ebeat.measure;
            current_beat = 0;
            is_even = (ebeat.measure % 2) == 0;
        } else {
            current_beat += 1;
        }

        let pi_id = find_beat_phrase_iteration_id(ebeat.time, phrase_iterations);

        let mut mask = BeatMask::empty();
        if is_first {
            mask |= BeatMask::FIRST_BEAT_OF_MEASURE;
            if is_even {
                mask |= BeatMask::EVEN_MEASURE;
            }
        }

        SngBeat {
            time: ms_to_sec(ebeat.time),
            measure: current_measure,
            beat: current_beat,
            phrase_iteration: pi_id as i32,
            mask,
        }
    }
}

/// Converts an XML Phrase to an SNG Phrase.
pub fn convert_phrase(
    arr: &InstrumentalArrangement,
    phrase_id: usize,
    phrase: &XmlPhrase,
) -> SngPhrase {
    // Count phrase iterations that reference this phrase
    let iteration_count = arr
        .phrase_iterations
        .iter()
        .filter(|pi| pi.phrase_id == phrase_id as u32)
        .count() as i32;

    SngPhrase {
        solo: phrase.solo,
        disparity: phrase.disparity,
        ignore: phrase.ignore,
        max_difficulty: phrase.max_difficulty as i32,
        iteration_count,
        name: string_to_bytes::<32>(&phrase.name),
    }
}

/// Converts an XML ChordTemplate to an SNG Chord.
pub fn convert_chord_template(arr: &InstrumentalArrangement, ct: &ChordTemplate) -> SngChord {
    let mask = if ct.display_name.ends_with("-arp") {
        SngChordMask::ARPEGGIO
    } else if ct.display_name.ends_with("-nop") {
        SngChordMask::NOP
    } else {
        SngChordMask::empty()
    };

    let is_bass = arr.meta.arrangement_properties.path_bass > 0;
    let notes = map_to_midi_notes(&ct.frets, &arr.meta.tuning.strings, arr.meta.capo, is_bass);

    SngChord {
        mask,
        frets: ct.frets,
        fingers: ct.fingers,
        notes,
        name: string_to_bytes::<32>(&ct.chord_name),
    }
}

/// Converts an XML BendValue to an SNG BendValue.
pub fn convert_bend_value(bv: &XmlBendValue) -> SngBendValue {
    SngBendValue {
        time: ms_to_sec(bv.time),
        step: bv.step as f32,
        unused: 0,
    }
}

/// Converts an XML PhraseIteration to an SNG PhraseIteration.
pub fn convert_phrase_iteration(
    pi_times: &[i32],
    index: usize,
    pi: &XmlPhraseIteration,
) -> SngPhraseIteration {
    let end_time = pi_times.get(index + 1).copied().unwrap_or(pi.end_time);

    let (easy, medium, hard) = crate::accu_data::get_hero_levels(pi);

    SngPhraseIteration {
        phrase_id: pi.phrase_id as i32,
        start_time: ms_to_sec(pi.time),
        end_time: ms_to_sec(end_time),
        difficulty: [easy, medium, hard],
    }
}

/// Converts an XML ArrangementEvent to an SNG Event.
pub fn convert_event(ev: &ArrangementEvent) -> SngEvent {
    SngEvent {
        time: ms_to_sec(ev.time),
        name: string_to_bytes::<256>(&ev.code),
    }
}

/// Converts an XML ToneChange to an SNG Tone.
pub fn convert_tone(tone: &ToneChange) -> SngTone {
    SngTone {
        time: ms_to_sec(tone.time),
        tone_id: tone.id,
    }
}

/// Creates an SNG DNA array from an arrangement's events.
pub fn create_dnas(arr: &InstrumentalArrangement) -> Vec<DNA> {
    arr.events
        .iter()
        .filter_map(|ev| {
            let id = match ev.code.as_str() {
                "dna_none" => Some(0),
                "dna_solo" => Some(1),
                "dna_riff" => Some(2),
                "dna_chord" => Some(3),
                _ => None,
            };
            id.map(|dna_id| DNA {
                time: ms_to_sec(ev.time),
                dna_id,
            })
        })
        .collect()
}

/// Converts an XML Section to an SNG Section.
pub fn convert_section(
    string_masks: &[Vec<i8>],
    arr: &InstrumentalArrangement,
    index: usize,
    section: &XmlSection,
) -> SngSection {
    let end_time = if index + 1 < arr.sections.len() {
        arr.sections[index + 1].start_time
    } else {
        arr.meta.song_length
    };

    let start_pi = find_phrase_iteration_id(section.start_time, &arr.phrase_iterations);

    // Find last PI where PI.Time < end_time, starting from start_pi + 1
    let end_pi = {
        let mut idx = start_pi as isize + 1;
        loop {
            if idx as usize >= arr.phrase_iterations.len()
                || arr.phrase_iterations[idx as usize].time >= end_time
            {
                idx -= 1;
                break;
            }
            idx += 1;
        }
        idx.max(start_pi as isize) as usize
    };

    // Build string mask [i8; 36]
    let mut string_mask = [0i8; 36];
    if let Some(sm) = string_masks.get(index) {
        let len = sm.len().min(36);
        string_mask[..len].copy_from_slice(&sm[..len]);
    }

    SngSection {
        name: string_to_bytes::<32>(&section.name),
        number: section.number,
        start_time: ms_to_sec(section.start_time),
        end_time: ms_to_sec(end_time),
        start_phrase_iteration_id: start_pi as i32,
        end_phrase_iteration_id: end_pi as i32,
        string_mask,
    }
}

/// Converts an XML Anchor to an SNG Anchor.
pub fn convert_anchor(
    notes: &[XmlNote],
    note_times: &[i32],
    level: &XmlLevel,
    arr: &InstrumentalArrangement,
    index: usize,
    anchor: &XmlAnchor,
) -> SngAnchor {
    let end_time = if index + 1 < level.anchors.len() {
        level.anchors[index + 1].time
    } else {
        arr.meta.song_length
    };

    let pi_id = find_phrase_iteration_id(anchor.time, &arr.phrase_iterations);

    // Find first/last note times within [anchor.time, end_time)
    let (first_note_time, last_note_time) =
        crate::utils::find_first_and_last_time(note_times, anchor.time, end_time)
            .map(|(first, last)| {
                let first_time = ms_to_sec(note_times[first]);
                // Last note time: if chord, use first chord note's sustain
                let last_sust = get_note_sustain_ms(notes, level, note_times[last]);
                let last_time = if note_times[last] + last_sust >= end_time {
                    f32::MAX
                } else {
                    ms_to_sec(note_times[last])
                };
                (first_time, last_time)
            })
            .unwrap_or((f32::MAX, f32::MIN_POSITIVE));

    SngAnchor {
        start_time: ms_to_sec(anchor.time),
        end_time: ms_to_sec(end_time),
        first_note_time,
        last_note_time,
        fret_id: anchor.fret,
        width: anchor.width,
        phrase_iteration_id: pi_id as i32,
    }
}

fn get_note_sustain_ms(notes: &[XmlNote], level: &XmlLevel, time: i32) -> i32 {
    // Check notes
    for n in notes {
        if n.time == time {
            return n.sustain;
        }
    }
    // Check chords
    for c in &level.chords {
        if c.time == time {
            return c.chord_notes.first().map(|cn| cn.sustain).unwrap_or(0);
        }
    }
    0
}

/// Converts an XML HandShape to an SNG FingerPrint.
pub fn convert_handshape(
    note_times: &[i32],
    entities: &[XmlEntity],
    hs: &HandShape,
) -> FingerPrint {
    let (first_note_time, last_note_time) =
        crate::utils::find_first_and_last_time(note_times, hs.start_time, hs.end_time)
            .map(|(first, last)| {
                let first_time = ms_to_sec(note_times[first]);
                let last_sust = get_entity_sustain(entities, last);
                let last_time = if note_times[last] + last_sust >= hs.end_time {
                    -1.0f32
                } else {
                    ms_to_sec(note_times[last])
                };
                (first_time, last_time)
            })
            .unwrap_or((-1.0f32, -1.0f32));

    FingerPrint {
        chord_id: hs.chord_id,
        start_time: ms_to_sec(hs.start_time),
        end_time: ms_to_sec(hs.end_time),
        first_note_time,
        last_note_time,
    }
}

fn get_entity_sustain(entities: &[XmlEntity], index: usize) -> i32 {
    match entities.get(index) {
        Some(XmlEntity::Note(n)) => n.sustain,
        Some(XmlEntity::Chord(c)) => c.chord_notes.first().map(|cn| cn.sustain).unwrap_or(0),
        None => 0,
    }
}

/// Creates SNG MetaData from the arrangement and accumulated data.
pub fn create_meta_data(
    accu: &crate::accu_data::AccuData,
    first_note_time: f32,
    arr: &InstrumentalArrangement,
) -> SngMetaData {
    let max_score = 100_000.0f64;
    let hard = accu.note_counts.hard as f64;
    let ignored = accu.note_counts.ignored as f64;

    let points_per_note = if hard > 0.0 {
        max_score / hard
    } else {
        max_score
    };

    let first_beat_length = if arr.ebeats.len() >= 2 {
        ms_to_sec(arr.ebeats[1].time - arr.ebeats[0].time)
    } else {
        0.0
    };

    let capo_fret_id = if arr.meta.capo <= 0 {
        -1i8
    } else {
        arr.meta.capo
    };

    let start_time = ms_to_sec(arr.meta.start_beat);

    SngMetaData {
        max_score,
        max_notes_and_chords: hard,
        max_notes_and_chords_real: (hard - ignored).max(0.0),
        points_per_note,
        first_beat_length,
        start_time,
        capo_fret_id,
        last_conversion_date_time: string_to_bytes::<32>(""),
        part: arr.meta.part as i16,
        song_length: ms_to_sec(arr.meta.song_length),
        tuning: arr.meta.tuning.strings.to_vec(),
        first_note_time,
        max_difficulty: arr.levels.len() as i32 - 1,
    }
}

/// An XML note or chord entity for level conversion.
#[derive(Clone, Debug)]
pub enum XmlEntity {
    Note(XmlNote),
    Chord(rocksmith2014_xml::Chord),
}

impl XmlEntity {
    pub fn time(&self) -> i32 {
        match self {
            XmlEntity::Note(n) => n.time,
            XmlEntity::Chord(c) => c.time,
        }
    }
}

/// Creates a sorted entity array from notes and chords in a level.
pub fn create_entity_array(level: &XmlLevel) -> Vec<XmlEntity> {
    let mut entities: Vec<XmlEntity> = level
        .notes
        .iter()
        .map(|n| XmlEntity::Note(n.clone()))
        .chain(level.chords.iter().map(|c| XmlEntity::Chord(c.clone())))
        .collect();
    entities.sort_by_key(|e| e.time());
    entities
}
