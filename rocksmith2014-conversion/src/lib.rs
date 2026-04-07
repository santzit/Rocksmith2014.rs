//! Rocksmith 2014 arrangement format conversion between SNG and XML.
//!
//! # SNG → XML
//! ```no_run
//! use rocksmith2014_sng::Sng;
//! use rocksmith2014_conversion::sng_to_xml;
//!
//! let data = std::fs::read("song.sng").unwrap();
//! let sng = Sng::read(&data).unwrap();
//! let arr = sng_to_xml(&sng);
//! println!("Levels: {}", arr.levels.len());
//! ```
//!
//! # XML → SNG
//! ```no_run
//! use rocksmith2014_xml::InstrumentalArrangement;
//! use rocksmith2014_conversion::xml_to_sng;
//!
//! let arr = rocksmith2014_xml::read_file("song.xml").unwrap();
//! let sng = xml_to_sng(&arr);
//! println!("Notes: {}", sng.levels.get(0).map(|l| l.notes.len()).unwrap_or(0));
//! ```

mod accu_data;
mod sng_to_xml;
mod utils;
mod xml_to_sng;
mod xml_to_sng_level;
mod xml_to_sng_note;

use rocksmith2014_sng::Sng;
use rocksmith2014_xml::InstrumentalArrangement;

pub use accu_data::{AccuData, NoteCounts};
pub use sng_to_xml::{
    convert_anchor as sng_convert_anchor, convert_beat as sng_convert_beat,
    convert_bend_value as sng_convert_bend_value, convert_chord as sng_convert_chord,
    convert_chord_template as sng_convert_chord_template, convert_event as sng_convert_event,
    convert_hand_shape as sng_convert_hand_shape, convert_level as sng_convert_level,
    convert_note as sng_convert_note, convert_phrase as sng_convert_phrase,
    convert_phrase_extra_info as sng_convert_phrase_extra_info,
    convert_phrase_iteration as sng_convert_phrase_iteration,
    convert_section as sng_convert_section, convert_tone as sng_convert_tone, sng_to_xml,
};
pub use xml_to_sng::{
    convert_anchor as xml_convert_anchor, convert_bend_value as xml_convert_bend_value,
    convert_chord_template as xml_convert_chord_template, convert_event as xml_convert_event,
    convert_handshape as xml_convert_handshape, convert_phrase as xml_convert_phrase,
    convert_phrase_iteration as xml_convert_phrase_iteration,
    convert_section as xml_convert_section, convert_tone as xml_convert_tone, make_beat_converter,
    map_to_midi_notes, to_midi_note, XmlEntity,
};
pub use xml_to_sng_level::convert_level as xml_convert_level;
pub use xml_to_sng_note::{flag_never, flag_on_anchor_change, FlagFn, NoteConverter};

/// Converts a full SNG arrangement to an XML InstrumentalArrangement.
pub fn sng_to_xml_full(sng: &Sng) -> InstrumentalArrangement {
    sng_to_xml(sng)
}

/// Converts an XML InstrumentalArrangement to an SNG arrangement.
pub fn xml_to_sng(arr: &InstrumentalArrangement) -> Sng {
    use xml_to_sng::{
        convert_chord_template, convert_event, convert_phrase, convert_phrase_iteration,
        convert_section, convert_tone, create_dnas, create_meta_data,
        create_phrase_iteration_times_array, make_beat_converter,
    };

    let mut accu = AccuData::init(arr);

    let beats = {
        let mut converter = make_beat_converter(arr);
        arr.ebeats.iter().map(|b| converter(b)).collect()
    };

    let phrases = arr
        .phrases
        .iter()
        .enumerate()
        .map(|(i, p)| convert_phrase(arr, i, p))
        .collect();

    let chord_templates = arr
        .chord_templates
        .iter()
        .map(|ct| convert_chord_template(arr, ct))
        .collect();

    let pi_times = create_phrase_iteration_times_array(arr);

    let phrase_iterations = arr
        .phrase_iterations
        .iter()
        .enumerate()
        .map(|(i, pi)| convert_phrase_iteration(&pi_times, i, pi))
        .collect();

    let phrase_extra_info = arr
        .phrase_properties
        .iter()
        .map(|pp| {
            use rocksmith2014_sng::PhraseExtraInfo;
            PhraseExtraInfo {
                phrase_id: pp.phrase_id,
                difficulty: pp.difficulty,
                empty: pp.empty,
                level_jump: pp.level_jump as i8,
                redundant: pp.redundant as i16,
            }
        })
        .collect();

    let new_linked_difficulties = vec![];

    let events = arr.events.iter().map(convert_event).collect();
    let tones = arr.tones.iter().map(convert_tone).collect();
    let dnas = create_dnas(arr);

    let sections: Vec<_> = arr
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| convert_section(&accu.string_masks.clone(), arr, i, s))
        .collect();

    let levels: Vec<_> = arr
        .levels
        .iter()
        .map(|l| xml_to_sng_level::convert_level(&mut accu, &pi_times, arr, l))
        .collect();

    // Recompute sections with final string masks
    let sections: Vec<_> = arr
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| convert_section(&accu.string_masks, arr, i, s))
        .collect();

    let first_note_time = levels
        .iter()
        .filter_map(|l| l.notes.first().map(|n| n.time))
        .fold(f32::MAX, f32::min);
    let first_note_time = if first_note_time == f32::MAX {
        0.0
    } else {
        first_note_time
    };

    let metadata = create_meta_data(&accu, first_note_time, arr);

    let mut sng = Sng {
        beats,
        phrases,
        chords: chord_templates,
        chord_notes: accu.chord_notes,
        phrase_iterations,
        phrase_extra_info,
        new_linked_difficulties,
        events,
        tones,
        dnas,
        sections,
        levels,
        metadata,
        ..Default::default()
    };

    sng
}
