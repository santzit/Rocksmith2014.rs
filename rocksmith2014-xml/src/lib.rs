//! Rust implementation of the Rocksmith 2014 XML arrangement format.
//!
//! This crate parses and generates the XML files that describe a Rocksmith 2014
//! instrumental arrangement: notes, chords, chord templates, anchors, phrases,
//! phrase iterations, sections, events, hand shapes, and arrangement metadata.
//!
//! # Reading an arrangement from XML
//!
//! ```no_run
//! use rocksmith2014_xml::read_file;
//!
//! let arr = read_file("song_lead.xml").unwrap();
//! println!("Title: {}", arr.meta.song_name);
//! println!("Levels: {}", arr.levels.len());
//! ```
//!
//! # Writing an arrangement back to XML
//!
//! ```no_run
//! use rocksmith2014_xml::{read_file, write_file};
//!
//! let arr = read_file("song_lead.xml").unwrap();
//! write_file(&arr, "song_lead_copy.xml").unwrap();
//! ```
//!
//! # Round-trip using in-memory strings
//!
//! ```
//! use rocksmith2014_xml::InstrumentalArrangement;
//!
//! let xml = r#"<?xml version="1.0" encoding="utf-8"?>
//! <song>
//!   <title>Test</title>
//!   <arrangement>Lead</arrangement>
//!   <part>1</part>
//!   <offset>0.000</offset>
//!   <centOffset>0</centOffset>
//!   <songLength>120.000</songLength>
//!   <startBeat>0.000</startBeat>
//!   <averageTempo>120.000</averageTempo>
//!   <version>7</version>
//!   <artistName>Artist</artistName>
//!   <artistNameSort>artist</artistNameSort>
//!   <albumName>Album</albumName>
//!   <albumNameSort>album</albumNameSort>
//!   <albumYear>2014</albumYear>
//!   <albumArt>art</albumArt>
//!   <crowdSpeed>1</crowdSpeed>
//!   <capo>0</capo>
//!   <tuning mi0="0" mi1="0" mi2="0" mi3="0" mi4="0" mi5="0" />
//!   <tones />
//!   <sections />
//!   <events />
//!   <ebeats />
//!   <phrases />
//!   <phraseIterations />
//!   <chordTemplates />
//!   <fretHandMuteTemplates />
//!   <linkedDiffs />
//!   <newLinkedDiffs />
//!   <phraseProperties />
//!   <levels />
//! </song>"#;
//!
//! let arr = InstrumentalArrangement::from_xml(xml).unwrap();
//! assert_eq!(arr.meta.song_name, "Test");
//! let roundtripped = arr.to_xml().unwrap();
//! assert!(roundtripped.contains("<title>Test</title>"));
//! ```

use std::fs;
use std::path::Path;
use quick_xml::{Reader, Writer};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event as XmlEvent};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("XML attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

fn time_from_str(s: &str) -> i32 {
    (s.parse::<f64>().unwrap_or(0.0) * 1000.0).round() as i32
}

fn time_to_str(ms: i32) -> String {
    format!("{:.3}", ms as f64 / 1000.0)
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct NoteMask: u16 {
        const LINK_NEXT      = 1 << 0;
        const ACCENT         = 1 << 1;
        const HAMMER_ON      = 1 << 2;
        const HARMONIC       = 1 << 3;
        const IGNORE         = 1 << 4;
        const FRET_HAND_MUTE = 1 << 5;
        const PALM_MUTE      = 1 << 6;
        const PULL_OFF       = 1 << 7;
        const TREMOLO        = 1 << 8;
        const PINCH_HARMONIC = 1 << 9;
        const PICK_DIRECTION = 1 << 10;
        const SLAP           = 1 << 11;
        const PLUCK          = 1 << 12;
        const RIGHT_HAND     = 1 << 13;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ChordMask: u8 {
        const FRET_HAND_MUTE = 1 << 0;
        const HIGH_DENSITY   = 1 << 1;
        const HOPO           = 1 << 2;
        const IGNORE         = 1 << 3;
        const LINK_NEXT      = 1 << 4;
        const PALM_MUTE      = 1 << 5;
        const ACCENT         = 1 << 6;
    }
}

fn get_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .and_then(|a| String::from_utf8(a.value.to_vec()).ok())
}

fn flag_from_attr(e: &BytesStart, name: &[u8]) -> bool {
    get_attr(e, name).as_deref() == Some("1")
}

fn parse_note_mask(e: &BytesStart) -> NoteMask {
    let mut mask = NoteMask::empty();
    if flag_from_attr(e, b"linkNext")      { mask |= NoteMask::LINK_NEXT; }
    if flag_from_attr(e, b"accent")        { mask |= NoteMask::ACCENT; }
    if flag_from_attr(e, b"hammerOn")      { mask |= NoteMask::HAMMER_ON; }
    if flag_from_attr(e, b"harmonic")      { mask |= NoteMask::HARMONIC; }
    if flag_from_attr(e, b"ignore")        { mask |= NoteMask::IGNORE; }
    if flag_from_attr(e, b"fretHandMute")  { mask |= NoteMask::FRET_HAND_MUTE; }
    if flag_from_attr(e, b"palmMute")      { mask |= NoteMask::PALM_MUTE; }
    if flag_from_attr(e, b"pullOff")       { mask |= NoteMask::PULL_OFF; }
    if flag_from_attr(e, b"tremolo")       { mask |= NoteMask::TREMOLO; }
    if flag_from_attr(e, b"pinchHarmonic") { mask |= NoteMask::PINCH_HARMONIC; }
    if flag_from_attr(e, b"pickDirection") { mask |= NoteMask::PICK_DIRECTION; }
    if flag_from_attr(e, b"slap")          { mask |= NoteMask::SLAP; }
    if flag_from_attr(e, b"pluck")         { mask |= NoteMask::PLUCK; }
    if flag_from_attr(e, b"rightHand")     { mask |= NoteMask::RIGHT_HAND; }
    mask
}

fn parse_chord_mask(e: &BytesStart) -> ChordMask {
    let mut mask = ChordMask::empty();
    if flag_from_attr(e, b"fretHandMute") { mask |= ChordMask::FRET_HAND_MUTE; }
    if flag_from_attr(e, b"highDensity")  { mask |= ChordMask::HIGH_DENSITY; }
    if flag_from_attr(e, b"hopo")         { mask |= ChordMask::HOPO; }
    if flag_from_attr(e, b"ignore")       { mask |= ChordMask::IGNORE; }
    if flag_from_attr(e, b"linkNext")     { mask |= ChordMask::LINK_NEXT; }
    if flag_from_attr(e, b"palmMute")     { mask |= ChordMask::PALM_MUTE; }
    if flag_from_attr(e, b"accent")       { mask |= ChordMask::ACCENT; }
    mask
}

// --- Data structures ---

#[derive(Debug, Clone, Default)]
pub struct Tuning {
    pub strings: [i16; 6],
}

#[derive(Debug, Clone, Default)]
pub struct ArrangementProperties {
    pub represent: u8,
    pub bonus_arr: u8,
    pub standard_tuning: u8,
    pub non_standard_chords: u8,
    pub barr_chords: u8,
    pub power_chords: u8,
    pub drop_d_power: u8,
    pub open_chords: u8,
    pub finger_picking: u8,
    pub pick_direction: u8,
    pub double_stops: u8,
    pub palm_mutes: u8,
    pub harmonics: u8,
    pub pinch_harmonics: u8,
    pub hopo: u8,
    pub tremolo: u8,
    pub slides: u8,
    pub unpitched_slides: u8,
    pub bends: u8,
    pub tapping: u8,
    pub vibrato: u8,
    pub fret_hand_mutes: u8,
    pub slap_pop: u8,
    pub two_finger_picking: u8,
    pub five_fret_chords: u8,
    pub chord_notes: u8,
    pub octaves: u8,
    pub sus_chords: u8,
    pub three_finger_chords: u8,
    pub rhythm_side: u8,
    pub solo: u8,
    pub path_lead: u8,
    pub path_rhythm: u8,
    pub path_bass: u8,
    pub routing_rules: u8,
    pub bass_pick: u8,
    pub synth_lead: u8,
    pub synth_bass: u8,
}

#[derive(Debug, Clone, Default)]
pub struct MetaData {
    pub song_name: String,
    pub arrangement: String,
    pub part: i32,
    pub offset: i32,
    pub cent_offset: f64,
    pub song_length: i32,
    pub last_conversion_date_time: String,
    pub start_beat: i32,
    pub average_tempo: f64,
    pub tuning: Tuning,
    pub capo: i8,
    pub artist_name: String,
    pub artist_name_sort: String,
    pub album_name: String,
    pub album_name_sort: String,
    pub album_year: i32,
    pub crowd_speed: i32,
    pub arrangement_properties: ArrangementProperties,
    pub tone_base: String,
    pub tone_a: String,
    pub tone_b: String,
    pub tone_c: String,
    pub tone_d: String,
    pub song_name_sort: String,
    pub internal_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Ebeat {
    pub time: i32,
    pub measure: i16,
}

#[derive(Debug, Clone, Default)]
pub struct Phrase {
    pub max_difficulty: u8,
    pub name: String,
    pub disparity: i8,
    pub ignore: i8,
    pub solo: i8,
}

#[derive(Debug, Clone, Default)]
pub struct PhraseIteration {
    pub time: i32,
    pub end_time: i32,
    pub phrase_id: u32,
    pub hero_levels: Option<Vec<HeroLevel>>,
}

#[derive(Debug, Clone, Default)]
pub struct HeroLevel {
    pub hero: i32,
    pub difficulty: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LinkedDiff {
    pub parent_id: i32,
    pub child_id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PhraseProperty {
    pub phrase_id: i32,
    pub redundant: i32,
    pub level_jump: i32,
    pub empty: i32,
    pub difficulty: i32,
}

#[derive(Debug, Clone)]
pub struct ChordTemplate {
    pub chord_name: String,
    pub display_name: String,
    pub fingers: [i8; 6],
    pub frets: [i8; 6],
}
impl Default for ChordTemplate {
    fn default() -> Self {
        Self {
            chord_name: String::new(),
            display_name: String::new(),
            fingers: [-1; 6],
            frets: [-1; 6],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BendValue {
    pub time: i32,
    pub step: f64,
    pub unk5: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Note {
    pub time: i32,
    pub string: i8,
    pub fret: i8,
    pub sustain: i32,
    pub vibrato: i8,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub tap: i8,
    pub pick_direction: i8,
    pub slap: i8,
    pub pluck: i8,
    pub max_bend: f64,
    pub mask: NoteMask,
    pub bend_values: Vec<BendValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ChordNote {
    pub string: i8,
    pub fret: i8,
    pub sustain: i32,
    pub vibrato: i8,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub bend_values: Vec<BendValue>,
    pub mask: NoteMask,
}

#[derive(Debug, Clone, Default)]
pub struct Chord {
    pub time: i32,
    pub chord_id: i32,
    pub sustain: i32,
    pub mask: ChordMask,
    pub chord_notes: Vec<ChordNote>,
}

#[derive(Debug, Clone, Default)]
pub struct Anchor {
    pub time: i32,
    pub end_time: i32,
    pub fret: i8,
    pub width: i32,
}

#[derive(Debug, Clone, Default)]
pub struct HandShape {
    pub chord_id: i32,
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Level {
    pub difficulty: i8,
    pub anchors: Vec<Anchor>,
    pub hand_shapes: Vec<HandShape>,
    pub notes: Vec<Note>,
    pub chords: Vec<Chord>,
}

#[derive(Debug, Clone, Default)]
pub struct ArrangementEvent {
    pub time: i32,
    pub code: String,
}

#[derive(Debug, Clone, Default)]
pub struct Section {
    pub name: String,
    pub number: i32,
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Debug, Clone, Default)]
pub struct ToneChange {
    pub time: i32,
    pub name: String,
    pub id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct InstrumentalArrangement {
    pub meta: MetaData,
    pub ebeats: Vec<Ebeat>,
    pub phrases: Vec<Phrase>,
    pub phrase_iterations: Vec<PhraseIteration>,
    pub linked_diffs: Vec<LinkedDiff>,
    pub phrase_properties: Vec<PhraseProperty>,
    pub chord_templates: Vec<ChordTemplate>,
    pub fret_hand_mute_templates: Vec<ChordTemplate>,
    pub events: Vec<ArrangementEvent>,
    pub sections: Vec<Section>,
    pub levels: Vec<Level>,
    pub tones: Vec<ToneChange>,
}

// ---- Parser helpers ----

fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String> {
    loop {
        match reader.read_event()? {
            XmlEvent::Text(t) => return Ok(t.unescape()?.into_owned()),
            XmlEvent::End(_) => return Ok(String::new()),
            XmlEvent::Eof => return Ok(String::new()),
            _ => {}
        }
    }
}

fn parse_bend_values(reader: &mut Reader<&[u8]>) -> Result<Vec<BendValue>> {
    let mut values = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"bendValue" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let step = get_attr(&e, b"step").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                values.push(BendValue { time, step, unk5: 0 });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"bendValue" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let step = get_attr(&e, b"step").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                values.push(BendValue { time, step, unk5: 0 });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"bendValues" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(values)
}

fn parse_note(reader: &mut Reader<&[u8]>, e: &BytesStart, is_start: bool) -> Result<Note> {
    let time = get_attr(e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
    let string = get_attr(e, b"string").and_then(|s| s.parse().ok()).unwrap_or(0);
    let fret = get_attr(e, b"fret").and_then(|s| s.parse().ok()).unwrap_or(0);
    let sustain = get_attr(e, b"sustain").map(|s| time_from_str(&s)).unwrap_or(0);
    let vibrato = get_attr(e, b"vibrato").and_then(|s| s.parse().ok()).unwrap_or(0);
    let slide_to = get_attr(e, b"slideTo").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let slide_unpitch_to = get_attr(e, b"slideUnpitchTo").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let left_hand = get_attr(e, b"leftHand").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let tap = get_attr(e, b"tap").and_then(|s| s.parse().ok()).unwrap_or(0);
    let pick_direction = get_attr(e, b"pickDirection").and_then(|s| s.parse().ok()).unwrap_or(0);
    let slap = get_attr(e, b"slap").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let pluck = get_attr(e, b"pluck").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let max_bend = get_attr(e, b"maxBend").and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let mask = parse_note_mask(e);

    let mut bend_values = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"bendValues" => {
                    bend_values = parse_bend_values(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"bendValues" => {
                    // empty bend values
                }
                XmlEvent::End(ce) if ce.name().as_ref() == b"note" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(Note {
        time,
        string,
        fret,
        sustain,
        vibrato,
        slide_to,
        slide_unpitch_to,
        left_hand,
        tap,
        pick_direction,
        slap,
        pluck,
        max_bend,
        mask,
        bend_values,
    })
}

fn parse_chord_note(reader: &mut Reader<&[u8]>, e: &BytesStart, is_start: bool) -> Result<ChordNote> {
    let string = get_attr(e, b"string").and_then(|s| s.parse().ok()).unwrap_or(0);
    let fret = get_attr(e, b"fret").and_then(|s| s.parse().ok()).unwrap_or(0);
    let sustain = get_attr(e, b"sustain").map(|s| time_from_str(&s)).unwrap_or(0);
    let vibrato = get_attr(e, b"vibrato").and_then(|s| s.parse().ok()).unwrap_or(0);
    let slide_to = get_attr(e, b"slideTo").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let slide_unpitch_to = get_attr(e, b"slideUnpitchTo").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let left_hand = get_attr(e, b"leftHand").and_then(|s| s.parse().ok()).unwrap_or(-1);
    let mask = parse_note_mask(e);

    let mut bend_values = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"bendValues" => {
                    bend_values = parse_bend_values(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"bendValues" => {}
                XmlEvent::End(ce) if ce.name().as_ref() == b"chordNote" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(ChordNote { string, fret, sustain, vibrato, slide_to, slide_unpitch_to, left_hand, bend_values, mask })
}

fn parse_chord_notes_list(reader: &mut Reader<&[u8]>) -> Result<Vec<ChordNote>> {
    let mut notes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chordNote" => {
                notes.push(parse_chord_note(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chordNote" => {
                notes.push(parse_chord_note(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"chordNotes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(notes)
}

fn parse_chord(reader: &mut Reader<&[u8]>, e: &BytesStart, is_start: bool) -> Result<Chord> {
    let time = get_attr(e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
    let chord_id = get_attr(e, b"chordId").and_then(|s| s.parse().ok()).unwrap_or(0);
    let sustain = get_attr(e, b"sustain").map(|s| time_from_str(&s)).unwrap_or(0);
    let mask = parse_chord_mask(e);

    let mut chord_notes = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"chordNotes" => {
                    chord_notes = parse_chord_notes_list(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"chordNotes" => {}
                XmlEvent::End(ce) if ce.name().as_ref() == b"chord" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(Chord { time, chord_id, sustain, mask, chord_notes })
}

fn parse_notes(reader: &mut Reader<&[u8]>) -> Result<Vec<Note>> {
    let mut notes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"note" => {
                notes.push(parse_note(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"note" => {
                notes.push(parse_note(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"notes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(notes)
}

fn parse_chords(reader: &mut Reader<&[u8]>) -> Result<Vec<Chord>> {
    let mut chords = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chord" => {
                chords.push(parse_chord(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chord" => {
                chords.push(parse_chord(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"chords" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(chords)
}

fn parse_anchors(reader: &mut Reader<&[u8]>) -> Result<Vec<Anchor>> {
    let mut anchors = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"anchor" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let fret = get_attr(&e, b"fret").and_then(|s| s.parse().ok()).unwrap_or(0);
                let width = get_attr(&e, b"width").and_then(|s| s.parse().ok()).unwrap_or(4);
                anchors.push(Anchor { time, end_time, fret, width });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"anchor" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let fret = get_attr(&e, b"fret").and_then(|s| s.parse().ok()).unwrap_or(0);
                let width = get_attr(&e, b"width").and_then(|s| s.parse().ok()).unwrap_or(4);
                anchors.push(Anchor { time, end_time, fret, width });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"anchors" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(anchors)
}

fn parse_hand_shapes(reader: &mut Reader<&[u8]>) -> Result<Vec<HandShape>> {
    let mut shapes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"handShape" => {
                let chord_id = get_attr(&e, b"chordId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let start_time = get_attr(&e, b"startTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                shapes.push(HandShape { chord_id, start_time, end_time });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"handShape" => {
                let chord_id = get_attr(&e, b"chordId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let start_time = get_attr(&e, b"startTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                shapes.push(HandShape { chord_id, start_time, end_time });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"handShapes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(shapes)
}

fn parse_level(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Level> {
    let difficulty = get_attr(e, b"difficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut anchors = vec![];
    let mut hand_shapes = vec![];
    let mut notes = vec![];
    let mut chords = vec![];

    loop {
        match reader.read_event()? {
            XmlEvent::Start(ce) => {
                match ce.name().as_ref() {
                    b"anchors" => { anchors = parse_anchors(reader)?; }
                    b"handShapes" => { hand_shapes = parse_hand_shapes(reader)?; }
                    b"notes" => { notes = parse_notes(reader)?; }
                    b"chords" => { chords = parse_chords(reader)?; }
                    _ => {
                        let end = ce.to_end().into_owned();
                        reader.read_to_end(end.name())?;
                    }
                }
            }
            XmlEvent::Empty(ce) => {
                match ce.name().as_ref() {
                    b"anchors" => {}
                    b"handShapes" => {}
                    b"notes" => {}
                    b"chords" => {}
                    _ => {}
                }
            }
            XmlEvent::End(ce) if ce.name().as_ref() == b"level" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }

    Ok(Level { difficulty, anchors, hand_shapes, notes, chords })
}

fn parse_levels(reader: &mut Reader<&[u8]>) -> Result<Vec<Level>> {
    let mut levels = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Start(e) if e.name().as_ref() == b"level" => {
                levels.push(parse_level(reader, &e)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"levels" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(levels)
}

fn parse_ebeats(reader: &mut Reader<&[u8]>) -> Result<Vec<Ebeat>> {
    let mut beats = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"ebeat" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let measure = get_attr(&e, b"measure").and_then(|s| s.parse().ok()).unwrap_or(-1);
                beats.push(Ebeat { time, measure });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"ebeat" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let measure = get_attr(&e, b"measure").and_then(|s| s.parse().ok()).unwrap_or(-1);
                beats.push(Ebeat { time, measure });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"ebeats" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(beats)
}

fn parse_phrases(reader: &mut Reader<&[u8]>) -> Result<Vec<Phrase>> {
    let mut phrases = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phrase" => {
                let max_difficulty = get_attr(&e, b"maxDifficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let disparity = get_attr(&e, b"disparity").and_then(|s| s.parse().ok()).unwrap_or(0);
                let ignore = get_attr(&e, b"ignore").and_then(|s| s.parse().ok()).unwrap_or(0);
                let solo = get_attr(&e, b"solo").and_then(|s| s.parse().ok()).unwrap_or(0);
                phrases.push(Phrase { max_difficulty, name, disparity, ignore, solo });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phrase" => {
                let max_difficulty = get_attr(&e, b"maxDifficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let disparity = get_attr(&e, b"disparity").and_then(|s| s.parse().ok()).unwrap_or(0);
                let ignore = get_attr(&e, b"ignore").and_then(|s| s.parse().ok()).unwrap_or(0);
                let solo = get_attr(&e, b"solo").and_then(|s| s.parse().ok()).unwrap_or(0);
                phrases.push(Phrase { max_difficulty, name, disparity, ignore, solo });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phrases" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(phrases)
}

fn parse_hero_levels(reader: &mut Reader<&[u8]>) -> Result<Vec<HeroLevel>> {
    let mut hero_levels = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"heroLevel" => {
                let hero = get_attr(&e, b"hero").and_then(|s| s.parse().ok()).unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                hero_levels.push(HeroLevel { hero, difficulty });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"heroLevel" => {
                let hero = get_attr(&e, b"hero").and_then(|s| s.parse().ok()).unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                hero_levels.push(HeroLevel { hero, difficulty });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"heroLevels" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(hero_levels)
}

fn parse_phrase_iterations(reader: &mut Reader<&[u8]>) -> Result<Vec<PhraseIteration>> {
    let mut iterations = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phraseIteration" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let phrase_id = get_attr(&e, b"phraseId").and_then(|s| s.parse().ok()).unwrap_or(0);
                iterations.push(PhraseIteration { time, end_time, phrase_id, hero_levels: None });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phraseIteration" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let phrase_id = get_attr(&e, b"phraseId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut hero_levels = None;
                loop {
                    match reader.read_event()? {
                        XmlEvent::Start(ce) if ce.name().as_ref() == b"heroLevels" => {
                            hero_levels = Some(parse_hero_levels(reader)?);
                        }
                        XmlEvent::Empty(ce) if ce.name().as_ref() == b"heroLevels" => {}
                        XmlEvent::End(ce) if ce.name().as_ref() == b"phraseIteration" => break,
                        XmlEvent::Eof => break,
                        _ => {}
                    }
                }
                iterations.push(PhraseIteration { time, end_time, phrase_id, hero_levels });
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phraseIterations" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(iterations)
}

fn parse_chord_templates(reader: &mut Reader<&[u8]>, end_tag: &[u8]) -> Result<Vec<ChordTemplate>> {
    let mut templates = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chordTemplate" => {
                templates.push(parse_chord_template_from_elem(&e));
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chordTemplate" => {
                templates.push(parse_chord_template_from_elem(&e));
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == end_tag => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(templates)
}

fn parse_chord_template_from_elem(e: &BytesStart) -> ChordTemplate {
    ChordTemplate {
        chord_name: get_attr(e, b"chordName").unwrap_or_default(),
        display_name: get_attr(e, b"displayName").unwrap_or_default(),
        fingers: [
            get_attr(e, b"finger0").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"finger1").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"finger2").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"finger3").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"finger4").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"finger5").and_then(|s| s.parse().ok()).unwrap_or(-1),
        ],
        frets: [
            get_attr(e, b"fret0").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"fret1").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"fret2").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"fret3").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"fret4").and_then(|s| s.parse().ok()).unwrap_or(-1),
            get_attr(e, b"fret5").and_then(|s| s.parse().ok()).unwrap_or(-1),
        ],
    }
}

fn parse_events(reader: &mut Reader<&[u8]>) -> Result<Vec<ArrangementEvent>> {
    let mut events = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"event" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let code = get_attr(&e, b"code").unwrap_or_default();
                events.push(ArrangementEvent { time, code });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"event" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let code = get_attr(&e, b"code").unwrap_or_default();
                events.push(ArrangementEvent { time, code });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"events" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(events)
}

fn parse_sections(reader: &mut Reader<&[u8]>) -> Result<Vec<Section>> {
    let mut sections = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"section" => {
                let name = get_attr(&e, b"name").unwrap_or_default();
                let number = get_attr(&e, b"number").and_then(|s| s.parse().ok()).unwrap_or(0);
                let start_time = get_attr(&e, b"startTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                sections.push(Section { name, number, start_time, end_time });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"section" => {
                let name = get_attr(&e, b"name").unwrap_or_default();
                let number = get_attr(&e, b"number").and_then(|s| s.parse().ok()).unwrap_or(0);
                let start_time = get_attr(&e, b"startTime").map(|s| time_from_str(&s)).unwrap_or(0);
                let end_time = get_attr(&e, b"endTime").map(|s| time_from_str(&s)).unwrap_or(0);
                sections.push(Section { name, number, start_time, end_time });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"sections" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(sections)
}

fn parse_tones(reader: &mut Reader<&[u8]>) -> Result<Vec<ToneChange>> {
    let mut tones = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"tone" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let id = get_attr(&e, b"id").and_then(|s| s.parse().ok()).unwrap_or(0);
                tones.push(ToneChange { time, name, id });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"tone" => {
                let time = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let id = get_attr(&e, b"id").and_then(|s| s.parse().ok()).unwrap_or(0);
                tones.push(ToneChange { time, name, id });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"tones" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(tones)
}

fn parse_linked_diffs(reader: &mut Reader<&[u8]>) -> Result<Vec<LinkedDiff>> {
    let mut diffs = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"linkedDiff" => {
                let parent_id = get_attr(&e, b"parentId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let child_id = get_attr(&e, b"childId").and_then(|s| s.parse().ok()).unwrap_or(0);
                diffs.push(LinkedDiff { parent_id, child_id });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"linkedDiff" => {
                let parent_id = get_attr(&e, b"parentId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let child_id = get_attr(&e, b"childId").and_then(|s| s.parse().ok()).unwrap_or(0);
                diffs.push(LinkedDiff { parent_id, child_id });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"linkedDiffs" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(diffs)
}

fn parse_phrase_properties(reader: &mut Reader<&[u8]>) -> Result<Vec<PhraseProperty>> {
    let mut props = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phraseProperty" => {
                let phrase_id = get_attr(&e, b"phraseId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let redundant = get_attr(&e, b"redundant").and_then(|s| s.parse().ok()).unwrap_or(0);
                let level_jump = get_attr(&e, b"levelJump").and_then(|s| s.parse().ok()).unwrap_or(0);
                let empty = get_attr(&e, b"empty").and_then(|s| s.parse().ok()).unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                props.push(PhraseProperty { phrase_id, redundant, level_jump, empty, difficulty });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phraseProperty" => {
                let phrase_id = get_attr(&e, b"phraseId").and_then(|s| s.parse().ok()).unwrap_or(0);
                let redundant = get_attr(&e, b"redundant").and_then(|s| s.parse().ok()).unwrap_or(0);
                let level_jump = get_attr(&e, b"levelJump").and_then(|s| s.parse().ok()).unwrap_or(0);
                let empty = get_attr(&e, b"empty").and_then(|s| s.parse().ok()).unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty").and_then(|s| s.parse().ok()).unwrap_or(0);
                props.push(PhraseProperty { phrase_id, redundant, level_jump, empty, difficulty });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phraseProperties" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(props)
}

fn parse_arrangement_properties(e: &BytesStart) -> ArrangementProperties {
    ArrangementProperties {
        represent: get_attr(e, b"represent").and_then(|s| s.parse().ok()).unwrap_or(0),
        bonus_arr: get_attr(e, b"bonusArr").and_then(|s| s.parse().ok()).unwrap_or(0),
        standard_tuning: get_attr(e, b"standardTuning").and_then(|s| s.parse().ok()).unwrap_or(1),
        non_standard_chords: get_attr(e, b"nonStandardChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        barr_chords: get_attr(e, b"barrChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        power_chords: get_attr(e, b"powerChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        drop_d_power: get_attr(e, b"dropDPower").and_then(|s| s.parse().ok()).unwrap_or(0),
        open_chords: get_attr(e, b"openChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        finger_picking: get_attr(e, b"fingerPicking").and_then(|s| s.parse().ok()).unwrap_or(0),
        pick_direction: get_attr(e, b"pickDirection").and_then(|s| s.parse().ok()).unwrap_or(0),
        double_stops: get_attr(e, b"doubleStops").and_then(|s| s.parse().ok()).unwrap_or(0),
        palm_mutes: get_attr(e, b"palmMutes").and_then(|s| s.parse().ok()).unwrap_or(0),
        harmonics: get_attr(e, b"harmonics").and_then(|s| s.parse().ok()).unwrap_or(0),
        pinch_harmonics: get_attr(e, b"pinchHarmonics").and_then(|s| s.parse().ok()).unwrap_or(0),
        hopo: get_attr(e, b"hopo").and_then(|s| s.parse().ok()).unwrap_or(0),
        tremolo: get_attr(e, b"tremolo").and_then(|s| s.parse().ok()).unwrap_or(0),
        slides: get_attr(e, b"slides").and_then(|s| s.parse().ok()).unwrap_or(0),
        unpitched_slides: get_attr(e, b"unpitchedSlides").and_then(|s| s.parse().ok()).unwrap_or(0),
        bends: get_attr(e, b"bends").and_then(|s| s.parse().ok()).unwrap_or(0),
        tapping: get_attr(e, b"tapping").and_then(|s| s.parse().ok()).unwrap_or(0),
        vibrato: get_attr(e, b"vibrato").and_then(|s| s.parse().ok()).unwrap_or(0),
        fret_hand_mutes: get_attr(e, b"fretHandMutes").and_then(|s| s.parse().ok()).unwrap_or(0),
        slap_pop: get_attr(e, b"slapPop").and_then(|s| s.parse().ok()).unwrap_or(0),
        two_finger_picking: get_attr(e, b"twoFingerPicking").and_then(|s| s.parse().ok()).unwrap_or(0),
        five_fret_chords: get_attr(e, b"fiveFretChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        chord_notes: get_attr(e, b"chordNotes").and_then(|s| s.parse().ok()).unwrap_or(0),
        octaves: get_attr(e, b"octaves").and_then(|s| s.parse().ok()).unwrap_or(0),
        sus_chords: get_attr(e, b"susChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        three_finger_chords: get_attr(e, b"threeFingerChords").and_then(|s| s.parse().ok()).unwrap_or(0),
        rhythm_side: get_attr(e, b"rhythmSide").and_then(|s| s.parse().ok()).unwrap_or(0),
        solo: get_attr(e, b"solo").and_then(|s| s.parse().ok()).unwrap_or(0),
        path_lead: get_attr(e, b"pathLead").and_then(|s| s.parse().ok()).unwrap_or(0),
        path_rhythm: get_attr(e, b"pathRhythm").and_then(|s| s.parse().ok()).unwrap_or(0),
        path_bass: get_attr(e, b"pathBass").and_then(|s| s.parse().ok()).unwrap_or(0),
        routing_rules: get_attr(e, b"routingRules").and_then(|s| s.parse().ok()).unwrap_or(0),
        bass_pick: get_attr(e, b"bassPick").and_then(|s| s.parse().ok()).unwrap_or(0),
        synth_lead: get_attr(e, b"synthLead").and_then(|s| s.parse().ok()).unwrap_or(0),
        synth_bass: get_attr(e, b"synthBass").and_then(|s| s.parse().ok()).unwrap_or(0),
    }
}

fn parse_tuning(e: &BytesStart) -> Tuning {
    Tuning {
        strings: [
            get_attr(e, b"string0").and_then(|s| s.parse().ok()).unwrap_or(0),
            get_attr(e, b"string1").and_then(|s| s.parse().ok()).unwrap_or(0),
            get_attr(e, b"string2").and_then(|s| s.parse().ok()).unwrap_or(0),
            get_attr(e, b"string3").and_then(|s| s.parse().ok()).unwrap_or(0),
            get_attr(e, b"string4").and_then(|s| s.parse().ok()).unwrap_or(0),
            get_attr(e, b"string5").and_then(|s| s.parse().ok()).unwrap_or(0),
        ],
    }
}

fn parse_song(reader: &mut Reader<&[u8]>) -> Result<InstrumentalArrangement> {
    let mut arr = InstrumentalArrangement::default();

    loop {
        match reader.read_event()? {
            XmlEvent::Start(e) => {
                match e.name().as_ref() {
                    b"title" => {
                        arr.meta.song_name = read_text_content(reader)?;
                    }
                    b"arrangement" => {
                        arr.meta.arrangement = read_text_content(reader)?;
                    }
                    b"part" => {
                        let s = read_text_content(reader)?;
                        arr.meta.part = s.parse().unwrap_or(1);
                    }
                    b"offset" => {
                        let s = read_text_content(reader)?;
                        arr.meta.offset = time_from_str(&s);
                    }
                    b"centOffset" => {
                        let s = read_text_content(reader)?;
                        arr.meta.cent_offset = s.parse().unwrap_or(0.0);
                    }
                    b"songLength" => {
                        let s = read_text_content(reader)?;
                        arr.meta.song_length = time_from_str(&s);
                    }
                    b"averageTempo" => {
                        let s = read_text_content(reader)?;
                        arr.meta.average_tempo = s.parse().unwrap_or(120.0);
                    }
                    b"lastConversionDateTime" => {
                        arr.meta.last_conversion_date_time = read_text_content(reader)?;
                    }
                    b"startBeat" => {
                        let s = read_text_content(reader)?;
                        arr.meta.start_beat = time_from_str(&s);
                    }
                    b"capo" => {
                        let s = read_text_content(reader)?;
                        arr.meta.capo = s.parse().unwrap_or(0);
                    }
                    b"artistName" => {
                        arr.meta.artist_name = read_text_content(reader)?;
                    }
                    b"artistNameSort" => {
                        arr.meta.artist_name_sort = read_text_content(reader)?;
                    }
                    b"albumName" => {
                        arr.meta.album_name = read_text_content(reader)?;
                    }
                    b"albumNameSort" => {
                        arr.meta.album_name_sort = read_text_content(reader)?;
                    }
                    b"albumYear" => {
                        let s = read_text_content(reader)?;
                        arr.meta.album_year = s.parse().unwrap_or(0);
                    }
                    b"crowdSpeed" => {
                        let s = read_text_content(reader)?;
                        arr.meta.crowd_speed = s.parse().unwrap_or(1);
                    }
                    b"tonebase" => {
                        arr.meta.tone_base = read_text_content(reader)?;
                    }
                    b"tonea" => {
                        arr.meta.tone_a = read_text_content(reader)?;
                    }
                    b"toneb" => {
                        arr.meta.tone_b = read_text_content(reader)?;
                    }
                    b"tonec" => {
                        arr.meta.tone_c = read_text_content(reader)?;
                    }
                    b"toned" => {
                        arr.meta.tone_d = read_text_content(reader)?;
                    }
                    b"songNameSort" => {
                        arr.meta.song_name_sort = read_text_content(reader)?;
                    }
                    b"internalName" => {
                        arr.meta.internal_name = read_text_content(reader)?;
                    }
                    b"ebeats" => {
                        arr.ebeats = parse_ebeats(reader)?;
                    }
                    b"phrases" => {
                        arr.phrases = parse_phrases(reader)?;
                    }
                    b"phraseIterations" => {
                        arr.phrase_iterations = parse_phrase_iterations(reader)?;
                    }
                    b"linkedDiffs" => {
                        arr.linked_diffs = parse_linked_diffs(reader)?;
                    }
                    b"phraseProperties" => {
                        arr.phrase_properties = parse_phrase_properties(reader)?;
                    }
                    b"chordTemplates" => {
                        arr.chord_templates = parse_chord_templates(reader, b"chordTemplates")?;
                    }
                    b"fretHandMuteTemplates" => {
                        arr.fret_hand_mute_templates = parse_chord_templates(reader, b"fretHandMuteTemplates")?;
                    }
                    b"events" => {
                        arr.events = parse_events(reader)?;
                    }
                    b"sections" => {
                        arr.sections = parse_sections(reader)?;
                    }
                    b"levels" => {
                        arr.levels = parse_levels(reader)?;
                    }
                    b"tones" => {
                        arr.tones = parse_tones(reader)?;
                    }
                    b"song" => {
                        // entering song - just continue
                    }
                    _ => {
                        let end = e.to_end().into_owned();
                        reader.read_to_end(end.name())?;
                    }
                }
            }
            XmlEvent::Empty(e) => {
                match e.name().as_ref() {
                    b"songLength" => {
                        arr.meta.song_length = get_attr(&e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
                    }
                    b"averageTempo" => {
                        arr.meta.average_tempo = get_attr(&e, b"bpm").and_then(|s| s.parse().ok()).unwrap_or(120.0);
                    }
                    b"tuning" => {
                        arr.meta.tuning = parse_tuning(&e);
                    }
                    b"arrangementProperties" => {
                        arr.meta.arrangement_properties = parse_arrangement_properties(&e);
                    }
                    b"ebeats" => {}
                    b"phrases" => {}
                    b"phraseIterations" => {}
                    b"linkedDiffs" => {}
                    b"phraseProperties" => {}
                    b"chordTemplates" => {}
                    b"fretHandMuteTemplates" => {}
                    b"events" => {}
                    b"sections" => {}
                    b"levels" => {}
                    b"tones" => {}
                    b"controls" => {}
                    _ => {}
                }
            }
            XmlEvent::End(e) if e.name().as_ref() == b"song" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }

    Ok(arr)
}

// ---- Writer helpers ----

fn write_flag(elem: &mut BytesStart, name: &str, mask: bool) {
    elem.push_attribute((name, if mask { "1" } else { "0" }));
}

fn write_note(writer: &mut Writer<Vec<u8>>, note: &Note) -> Result<()> {
    let mut elem = BytesStart::new("note");
    elem.push_attribute(("time", time_to_str(note.time).as_str()));
    elem.push_attribute(("sustain", time_to_str(note.sustain).as_str()));
    elem.push_attribute(("string", note.string.to_string().as_str()));
    elem.push_attribute(("fret", note.fret.to_string().as_str()));
    elem.push_attribute(("leftHand", note.left_hand.to_string().as_str()));
    elem.push_attribute(("slideTo", note.slide_to.to_string().as_str()));
    elem.push_attribute(("slideUnpitchTo", note.slide_unpitch_to.to_string().as_str()));
    elem.push_attribute(("tap", note.tap.to_string().as_str()));
    elem.push_attribute(("pickDirection", note.pick_direction.to_string().as_str()));
    elem.push_attribute(("slap", note.slap.to_string().as_str()));
    elem.push_attribute(("pluck", note.pluck.to_string().as_str()));
    elem.push_attribute(("vibrato", note.vibrato.to_string().as_str()));
    elem.push_attribute(("maxBend", note.max_bend.to_string().as_str()));
    write_flag(&mut elem, "linkNext",      note.mask.contains(NoteMask::LINK_NEXT));
    write_flag(&mut elem, "accent",        note.mask.contains(NoteMask::ACCENT));
    write_flag(&mut elem, "hammerOn",      note.mask.contains(NoteMask::HAMMER_ON));
    write_flag(&mut elem, "harmonic",      note.mask.contains(NoteMask::HARMONIC));
    write_flag(&mut elem, "ignore",        note.mask.contains(NoteMask::IGNORE));
    write_flag(&mut elem, "fretHandMute",  note.mask.contains(NoteMask::FRET_HAND_MUTE));
    write_flag(&mut elem, "palmMute",      note.mask.contains(NoteMask::PALM_MUTE));
    write_flag(&mut elem, "pullOff",       note.mask.contains(NoteMask::PULL_OFF));
    write_flag(&mut elem, "tremolo",       note.mask.contains(NoteMask::TREMOLO));
    write_flag(&mut elem, "pinchHarmonic", note.mask.contains(NoteMask::PINCH_HARMONIC));
    write_flag(&mut elem, "rightHand",     note.mask.contains(NoteMask::RIGHT_HAND));

    if note.bend_values.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut bv_elem = BytesStart::new("bendValues");
        bv_elem.push_attribute(("count", note.bend_values.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(bv_elem))?;
        for bv in &note.bend_values {
            let mut bve = BytesStart::new("bendValue");
            bve.push_attribute(("time", time_to_str(bv.time).as_str()));
            bve.push_attribute(("step", bv.step.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(bve))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("bendValues")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("note")))?;
    }
    Ok(())
}

fn write_chord_note(writer: &mut Writer<Vec<u8>>, cn: &ChordNote) -> Result<()> {
    let mut elem = BytesStart::new("chordNote");
    elem.push_attribute(("string", cn.string.to_string().as_str()));
    elem.push_attribute(("fret", cn.fret.to_string().as_str()));
    elem.push_attribute(("sustain", time_to_str(cn.sustain).as_str()));
    elem.push_attribute(("vibrato", cn.vibrato.to_string().as_str()));
    elem.push_attribute(("slideTo", cn.slide_to.to_string().as_str()));
    elem.push_attribute(("slideUnpitchTo", cn.slide_unpitch_to.to_string().as_str()));
    elem.push_attribute(("leftHand", cn.left_hand.to_string().as_str()));
    write_flag(&mut elem, "linkNext",      cn.mask.contains(NoteMask::LINK_NEXT));
    write_flag(&mut elem, "accent",        cn.mask.contains(NoteMask::ACCENT));
    write_flag(&mut elem, "hammerOn",      cn.mask.contains(NoteMask::HAMMER_ON));
    write_flag(&mut elem, "harmonic",      cn.mask.contains(NoteMask::HARMONIC));
    write_flag(&mut elem, "ignore",        cn.mask.contains(NoteMask::IGNORE));
    write_flag(&mut elem, "fretHandMute",  cn.mask.contains(NoteMask::FRET_HAND_MUTE));
    write_flag(&mut elem, "palmMute",      cn.mask.contains(NoteMask::PALM_MUTE));
    write_flag(&mut elem, "pullOff",       cn.mask.contains(NoteMask::PULL_OFF));
    write_flag(&mut elem, "tremolo",       cn.mask.contains(NoteMask::TREMOLO));
    write_flag(&mut elem, "pinchHarmonic", cn.mask.contains(NoteMask::PINCH_HARMONIC));
    write_flag(&mut elem, "rightHand",     cn.mask.contains(NoteMask::RIGHT_HAND));

    if cn.bend_values.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut bv_elem = BytesStart::new("bendValues");
        bv_elem.push_attribute(("count", cn.bend_values.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(bv_elem))?;
        for bv in &cn.bend_values {
            let mut bve = BytesStart::new("bendValue");
            bve.push_attribute(("time", time_to_str(bv.time).as_str()));
            bve.push_attribute(("step", bv.step.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(bve))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("bendValues")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("chordNote")))?;
    }
    Ok(())
}

fn write_chord(writer: &mut Writer<Vec<u8>>, chord: &Chord) -> Result<()> {
    let mut elem = BytesStart::new("chord");
    elem.push_attribute(("time", time_to_str(chord.time).as_str()));
    elem.push_attribute(("chordId", chord.chord_id.to_string().as_str()));
    if chord.sustain > 0 {
        elem.push_attribute(("sustain", time_to_str(chord.sustain).as_str()));
    }
    write_flag(&mut elem, "fretHandMute", chord.mask.contains(ChordMask::FRET_HAND_MUTE));
    write_flag(&mut elem, "highDensity",  chord.mask.contains(ChordMask::HIGH_DENSITY));
    write_flag(&mut elem, "hopo",         chord.mask.contains(ChordMask::HOPO));
    write_flag(&mut elem, "ignore",       chord.mask.contains(ChordMask::IGNORE));
    write_flag(&mut elem, "linkNext",     chord.mask.contains(ChordMask::LINK_NEXT));
    write_flag(&mut elem, "palmMute",     chord.mask.contains(ChordMask::PALM_MUTE));
    write_flag(&mut elem, "accent",       chord.mask.contains(ChordMask::ACCENT));

    if chord.chord_notes.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut cn_elem = BytesStart::new("chordNotes");
        cn_elem.push_attribute(("count", chord.chord_notes.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(cn_elem))?;
        for cn in &chord.chord_notes {
            write_chord_note(writer, cn)?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("chordNotes")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("chord")))?;
    }
    Ok(())
}

fn write_text_element(writer: &mut Writer<Vec<u8>>, tag: &str, content: &str) -> Result<()> {
    writer.write_event(XmlEvent::Start(BytesStart::new(tag)))?;
    writer.write_event(XmlEvent::Text(BytesText::new(content)))?;
    writer.write_event(XmlEvent::End(BytesEnd::new(tag)))?;
    Ok(())
}

impl InstrumentalArrangement {
    pub fn from_xml(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        loop {
            match reader.read_event()? {
                XmlEvent::Start(e) if e.name().as_ref() == b"song" => {
                    return parse_song(&mut reader);
                }
                XmlEvent::Eof => break,
                _ => {}
            }
        }

        Ok(InstrumentalArrangement::default())
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        let song_start = BytesStart::new("song");
        writer.write_event(XmlEvent::Start(song_start))?;

        write_text_element(&mut writer, "title", &self.meta.song_name)?;
        write_text_element(&mut writer, "arrangement", &self.meta.arrangement)?;
        write_text_element(&mut writer, "part", &self.meta.part.to_string())?;
        write_text_element(&mut writer, "offset", &format!("{:.3}", self.meta.offset as f64 / 1000.0))?;
        write_text_element(&mut writer, "centOffset", &self.meta.cent_offset.to_string())?;

        let mut song_length_elem = BytesStart::new("songLength");
        song_length_elem.push_attribute(("time", time_to_str(self.meta.song_length).as_str()));
        writer.write_event(XmlEvent::Empty(song_length_elem))?;

        write_text_element(&mut writer, "lastConversionDateTime", &self.meta.last_conversion_date_time)?;
        write_text_element(&mut writer, "startBeat", &time_to_str(self.meta.start_beat))?;

        let mut avg_tempo_elem = BytesStart::new("averageTempo");
        avg_tempo_elem.push_attribute(("bpm", format!("{:.3}", self.meta.average_tempo).as_str()));
        writer.write_event(XmlEvent::Empty(avg_tempo_elem))?;

        let mut tuning_elem = BytesStart::new("tuning");
        tuning_elem.push_attribute(("string0", self.meta.tuning.strings[0].to_string().as_str()));
        tuning_elem.push_attribute(("string1", self.meta.tuning.strings[1].to_string().as_str()));
        tuning_elem.push_attribute(("string2", self.meta.tuning.strings[2].to_string().as_str()));
        tuning_elem.push_attribute(("string3", self.meta.tuning.strings[3].to_string().as_str()));
        tuning_elem.push_attribute(("string4", self.meta.tuning.strings[4].to_string().as_str()));
        tuning_elem.push_attribute(("string5", self.meta.tuning.strings[5].to_string().as_str()));
        writer.write_event(XmlEvent::Empty(tuning_elem))?;

        write_text_element(&mut writer, "capo", &self.meta.capo.to_string())?;
        write_text_element(&mut writer, "artistName", &self.meta.artist_name)?;
        write_text_element(&mut writer, "artistNameSort", &self.meta.artist_name_sort)?;
        write_text_element(&mut writer, "albumName", &self.meta.album_name)?;
        write_text_element(&mut writer, "albumNameSort", &self.meta.album_name_sort)?;
        write_text_element(&mut writer, "albumYear", &self.meta.album_year.to_string())?;
        write_text_element(&mut writer, "crowdSpeed", &self.meta.crowd_speed.to_string())?;

        // arrangementProperties
        let ap = &self.meta.arrangement_properties;
        let mut ap_elem = BytesStart::new("arrangementProperties");
        ap_elem.push_attribute(("represent", ap.represent.to_string().as_str()));
        ap_elem.push_attribute(("bonusArr", ap.bonus_arr.to_string().as_str()));
        ap_elem.push_attribute(("standardTuning", ap.standard_tuning.to_string().as_str()));
        ap_elem.push_attribute(("nonStandardChords", ap.non_standard_chords.to_string().as_str()));
        ap_elem.push_attribute(("barrChords", ap.barr_chords.to_string().as_str()));
        ap_elem.push_attribute(("powerChords", ap.power_chords.to_string().as_str()));
        ap_elem.push_attribute(("dropDPower", ap.drop_d_power.to_string().as_str()));
        ap_elem.push_attribute(("openChords", ap.open_chords.to_string().as_str()));
        ap_elem.push_attribute(("fingerPicking", ap.finger_picking.to_string().as_str()));
        ap_elem.push_attribute(("pickDirection", ap.pick_direction.to_string().as_str()));
        ap_elem.push_attribute(("doubleStops", ap.double_stops.to_string().as_str()));
        ap_elem.push_attribute(("palmMutes", ap.palm_mutes.to_string().as_str()));
        ap_elem.push_attribute(("harmonics", ap.harmonics.to_string().as_str()));
        ap_elem.push_attribute(("pinchHarmonics", ap.pinch_harmonics.to_string().as_str()));
        ap_elem.push_attribute(("hopo", ap.hopo.to_string().as_str()));
        ap_elem.push_attribute(("tremolo", ap.tremolo.to_string().as_str()));
        ap_elem.push_attribute(("slides", ap.slides.to_string().as_str()));
        ap_elem.push_attribute(("unpitchedSlides", ap.unpitched_slides.to_string().as_str()));
        ap_elem.push_attribute(("bends", ap.bends.to_string().as_str()));
        ap_elem.push_attribute(("tapping", ap.tapping.to_string().as_str()));
        ap_elem.push_attribute(("vibrato", ap.vibrato.to_string().as_str()));
        ap_elem.push_attribute(("fretHandMutes", ap.fret_hand_mutes.to_string().as_str()));
        ap_elem.push_attribute(("slapPop", ap.slap_pop.to_string().as_str()));
        ap_elem.push_attribute(("twoFingerPicking", ap.two_finger_picking.to_string().as_str()));
        ap_elem.push_attribute(("fiveFretChords", ap.five_fret_chords.to_string().as_str()));
        ap_elem.push_attribute(("chordNotes", ap.chord_notes.to_string().as_str()));
        ap_elem.push_attribute(("octaves", ap.octaves.to_string().as_str()));
        ap_elem.push_attribute(("susChords", ap.sus_chords.to_string().as_str()));
        ap_elem.push_attribute(("threeFingerChords", ap.three_finger_chords.to_string().as_str()));
        ap_elem.push_attribute(("rhythmSide", ap.rhythm_side.to_string().as_str()));
        ap_elem.push_attribute(("solo", ap.solo.to_string().as_str()));
        ap_elem.push_attribute(("pathLead", ap.path_lead.to_string().as_str()));
        ap_elem.push_attribute(("pathRhythm", ap.path_rhythm.to_string().as_str()));
        ap_elem.push_attribute(("pathBass", ap.path_bass.to_string().as_str()));
        ap_elem.push_attribute(("routingRules", ap.routing_rules.to_string().as_str()));
        writer.write_event(XmlEvent::Empty(ap_elem))?;

        write_text_element(&mut writer, "tonebase", &self.meta.tone_base)?;
        write_text_element(&mut writer, "tonea", &self.meta.tone_a)?;
        write_text_element(&mut writer, "toneb", &self.meta.tone_b)?;
        write_text_element(&mut writer, "tonec", &self.meta.tone_c)?;
        write_text_element(&mut writer, "toned", &self.meta.tone_d)?;

        // tones
        let mut tones_elem = BytesStart::new("tones");
        tones_elem.push_attribute(("count", self.tones.len().to_string().as_str()));
        if self.tones.is_empty() {
            writer.write_event(XmlEvent::Empty(tones_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(tones_elem))?;
            for tone in &self.tones {
                let mut te = BytesStart::new("tone");
                te.push_attribute(("time", time_to_str(tone.time).as_str()));
                te.push_attribute(("name", tone.name.as_str()));
                te.push_attribute(("id", tone.id.to_string().as_str()));
                writer.write_event(XmlEvent::Empty(te))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("tones")))?;
        }

        // ebeats
        let mut ebeats_elem = BytesStart::new("ebeats");
        ebeats_elem.push_attribute(("count", self.ebeats.len().to_string().as_str()));
        if self.ebeats.is_empty() {
            writer.write_event(XmlEvent::Empty(ebeats_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(ebeats_elem))?;
            for beat in &self.ebeats {
                let mut be = BytesStart::new("ebeat");
                be.push_attribute(("time", time_to_str(beat.time).as_str()));
                be.push_attribute(("measure", beat.measure.to_string().as_str()));
                writer.write_event(XmlEvent::Empty(be))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("ebeats")))?;
        }

        // phrases
        let mut phrases_elem = BytesStart::new("phrases");
        phrases_elem.push_attribute(("count", self.phrases.len().to_string().as_str()));
        if self.phrases.is_empty() {
            writer.write_event(XmlEvent::Empty(phrases_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(phrases_elem))?;
            for phrase in &self.phrases {
                let mut pe = BytesStart::new("phrase");
                pe.push_attribute(("maxDifficulty", phrase.max_difficulty.to_string().as_str()));
                pe.push_attribute(("name", phrase.name.as_str()));
                pe.push_attribute(("disparity", phrase.disparity.to_string().as_str()));
                pe.push_attribute(("ignore", phrase.ignore.to_string().as_str()));
                pe.push_attribute(("solo", phrase.solo.to_string().as_str()));
                writer.write_event(XmlEvent::Empty(pe))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("phrases")))?;
        }

        // phraseIterations
        let mut pi_elem = BytesStart::new("phraseIterations");
        pi_elem.push_attribute(("count", self.phrase_iterations.len().to_string().as_str()));
        if self.phrase_iterations.is_empty() {
            writer.write_event(XmlEvent::Empty(pi_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(pi_elem))?;
            for pi in &self.phrase_iterations {
                let mut pie = BytesStart::new("phraseIteration");
                pie.push_attribute(("time", time_to_str(pi.time).as_str()));
                pie.push_attribute(("endTime", time_to_str(pi.end_time).as_str()));
                pie.push_attribute(("phraseId", pi.phrase_id.to_string().as_str()));
                pie.push_attribute(("variation", ""));  // variation field removed from struct; write empty for XML compatibility
                let hero_levels_nonempty = pi.hero_levels.as_ref().map_or(false, |v| !v.is_empty());
                if !hero_levels_nonempty {
                    writer.write_event(XmlEvent::Empty(pie))?;
                } else {
                    writer.write_event(XmlEvent::Start(pie))?;
                    let hls = pi.hero_levels.as_ref().unwrap();
                    let mut hl_elem = BytesStart::new("heroLevels");
                    hl_elem.push_attribute(("count", hls.len().to_string().as_str()));
                    writer.write_event(XmlEvent::Start(hl_elem))?;
                    for hl in hls {
                        let mut hle = BytesStart::new("heroLevel");
                        hle.push_attribute(("hero", hl.hero.to_string().as_str()));
                        hle.push_attribute(("difficulty", hl.difficulty.to_string().as_str()));
                        writer.write_event(XmlEvent::Empty(hle))?;
                    }
                    writer.write_event(XmlEvent::End(BytesEnd::new("heroLevels")))?;
                    writer.write_event(XmlEvent::End(BytesEnd::new("phraseIteration")))?;
                }
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("phraseIterations")))?;
        }

        // linkedDiffs
        let mut ld_elem = BytesStart::new("linkedDiffs");
        ld_elem.push_attribute(("count", self.linked_diffs.len().to_string().as_str()));
        writer.write_event(XmlEvent::Empty(ld_elem))?;

        // phraseProperties
        let mut pp_elem = BytesStart::new("phraseProperties");
        pp_elem.push_attribute(("count", self.phrase_properties.len().to_string().as_str()));
        writer.write_event(XmlEvent::Empty(pp_elem))?;

        // chordTemplates
        let mut ct_elem = BytesStart::new("chordTemplates");
        ct_elem.push_attribute(("count", self.chord_templates.len().to_string().as_str()));
        if self.chord_templates.is_empty() {
            writer.write_event(XmlEvent::Empty(ct_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(ct_elem))?;
            for ct in &self.chord_templates {
                let mut cte = BytesStart::new("chordTemplate");
                cte.push_attribute(("chordName", ct.chord_name.as_str()));
                cte.push_attribute(("displayName", ct.display_name.as_str()));
                cte.push_attribute(("finger0", ct.fingers[0].to_string().as_str()));
                cte.push_attribute(("finger1", ct.fingers[1].to_string().as_str()));
                cte.push_attribute(("finger2", ct.fingers[2].to_string().as_str()));
                cte.push_attribute(("finger3", ct.fingers[3].to_string().as_str()));
                cte.push_attribute(("finger4", ct.fingers[4].to_string().as_str()));
                cte.push_attribute(("finger5", ct.fingers[5].to_string().as_str()));
                cte.push_attribute(("fret0", ct.frets[0].to_string().as_str()));
                cte.push_attribute(("fret1", ct.frets[1].to_string().as_str()));
                cte.push_attribute(("fret2", ct.frets[2].to_string().as_str()));
                cte.push_attribute(("fret3", ct.frets[3].to_string().as_str()));
                cte.push_attribute(("fret4", ct.frets[4].to_string().as_str()));
                cte.push_attribute(("fret5", ct.frets[5].to_string().as_str()));
                writer.write_event(XmlEvent::Empty(cte))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("chordTemplates")))?;
        }

        // fretHandMuteTemplates
        let mut fhmt_elem = BytesStart::new("fretHandMuteTemplates");
        fhmt_elem.push_attribute(("count", self.fret_hand_mute_templates.len().to_string().as_str()));
        writer.write_event(XmlEvent::Empty(fhmt_elem))?;

        // controls
        let mut controls_elem = BytesStart::new("controls");
        controls_elem.push_attribute(("count", "0"));
        writer.write_event(XmlEvent::Empty(controls_elem))?;

        // events
        let mut events_elem = BytesStart::new("events");
        events_elem.push_attribute(("count", self.events.len().to_string().as_str()));
        if self.events.is_empty() {
            writer.write_event(XmlEvent::Empty(events_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(events_elem))?;
            for ev in &self.events {
                let mut eve = BytesStart::new("event");
                eve.push_attribute(("time", time_to_str(ev.time).as_str()));
                eve.push_attribute(("code", ev.code.as_str()));
                writer.write_event(XmlEvent::Empty(eve))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("events")))?;
        }

        // sections
        let mut sections_elem = BytesStart::new("sections");
        sections_elem.push_attribute(("count", self.sections.len().to_string().as_str()));
        if self.sections.is_empty() {
            writer.write_event(XmlEvent::Empty(sections_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(sections_elem))?;
            for sec in &self.sections {
                let mut se = BytesStart::new("section");
                se.push_attribute(("name", sec.name.as_str()));
                se.push_attribute(("number", sec.number.to_string().as_str()));
                se.push_attribute(("startTime", time_to_str(sec.start_time).as_str()));
                se.push_attribute(("endTime", time_to_str(sec.end_time).as_str()));
                writer.write_event(XmlEvent::Empty(se))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("sections")))?;
        }

        // levels
        let mut levels_elem = BytesStart::new("levels");
        levels_elem.push_attribute(("count", self.levels.len().to_string().as_str()));
        if self.levels.is_empty() {
            writer.write_event(XmlEvent::Empty(levels_elem))?;
        } else {
            writer.write_event(XmlEvent::Start(levels_elem))?;
            for level in &self.levels {
                let mut le = BytesStart::new("level");
                le.push_attribute(("difficulty", level.difficulty.to_string().as_str()));
                writer.write_event(XmlEvent::Start(le))?;

                // anchors
                let mut anch_elem = BytesStart::new("anchors");
                anch_elem.push_attribute(("count", level.anchors.len().to_string().as_str()));
                if level.anchors.is_empty() {
                    writer.write_event(XmlEvent::Empty(anch_elem))?;
                } else {
                    writer.write_event(XmlEvent::Start(anch_elem))?;
                    for anch in &level.anchors {
                        let mut ae = BytesStart::new("anchor");
                        ae.push_attribute(("time", time_to_str(anch.time).as_str()));
                        ae.push_attribute(("endTime", time_to_str(anch.end_time).as_str()));
                        ae.push_attribute(("fret", anch.fret.to_string().as_str()));
                        ae.push_attribute(("width", anch.width.to_string().as_str()));
                        writer.write_event(XmlEvent::Empty(ae))?;
                    }
                    writer.write_event(XmlEvent::End(BytesEnd::new("anchors")))?;
                }

                // handShapes
                let mut hs_elem = BytesStart::new("handShapes");
                hs_elem.push_attribute(("count", level.hand_shapes.len().to_string().as_str()));
                if level.hand_shapes.is_empty() {
                    writer.write_event(XmlEvent::Empty(hs_elem))?;
                } else {
                    writer.write_event(XmlEvent::Start(hs_elem))?;
                    for hs in &level.hand_shapes {
                        let mut hse = BytesStart::new("handShape");
                        hse.push_attribute(("chordId", hs.chord_id.to_string().as_str()));
                        hse.push_attribute(("startTime", time_to_str(hs.start_time).as_str()));
                        hse.push_attribute(("endTime", time_to_str(hs.end_time).as_str()));
                        writer.write_event(XmlEvent::Empty(hse))?;
                    }
                    writer.write_event(XmlEvent::End(BytesEnd::new("handShapes")))?;
                }

                // notes
                let mut notes_elem = BytesStart::new("notes");
                notes_elem.push_attribute(("count", level.notes.len().to_string().as_str()));
                if level.notes.is_empty() {
                    writer.write_event(XmlEvent::Empty(notes_elem))?;
                } else {
                    writer.write_event(XmlEvent::Start(notes_elem))?;
                    for note in &level.notes {
                        write_note(&mut writer, note)?;
                    }
                    writer.write_event(XmlEvent::End(BytesEnd::new("notes")))?;
                }

                // chords
                let mut chords_elem = BytesStart::new("chords");
                chords_elem.push_attribute(("count", level.chords.len().to_string().as_str()));
                if level.chords.is_empty() {
                    writer.write_event(XmlEvent::Empty(chords_elem))?;
                } else {
                    writer.write_event(XmlEvent::Start(chords_elem))?;
                    for chord in &level.chords {
                        write_chord(&mut writer, chord)?;
                    }
                    writer.write_event(XmlEvent::End(BytesEnd::new("chords")))?;
                }

                writer.write_event(XmlEvent::End(BytesEnd::new("level")))?;
            }
            writer.write_event(XmlEvent::End(BytesEnd::new("levels")))?;
        }

        writer.write_event(XmlEvent::End(BytesEnd::new("song")))?;

        Ok(String::from_utf8(writer.into_inner())?)
    }
}

pub fn read_file(path: impl AsRef<Path>) -> Result<InstrumentalArrangement> {
    let xml = fs::read_to_string(path)?;
    InstrumentalArrangement::from_xml(&xml)
}

pub fn write_file(arr: &InstrumentalArrangement, path: impl AsRef<Path>) -> Result<()> {
    let xml = arr.to_xml()?;
    fs::write(path, xml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<song>
  <title>Test Song</title>
  <arrangement>Lead</arrangement>
  <part>1</part>
  <offset>0.000</offset>
  <centOffset>0</centOffset>
  <songLength time="120.000" />
  <lastConversionDateTime>2023-01-01</lastConversionDateTime>
  <startBeat>0.000</startBeat>
  <averageTempo bpm="120.000" />
  <tuning string0="0" string1="0" string2="0" string3="0" string4="0" string5="0" />
  <capo>0</capo>
  <artistName>Test Artist</artistName>
  <albumName>Test Album</albumName>
  <albumYear>2023</albumYear>
  <crowdSpeed>1</crowdSpeed>
  <arrangementProperties represent="1" bonusArr="0" standardTuning="1" nonStandardChords="0" barrChords="0" powerChords="0" dropDPower="0" openChords="0" fingerPicking="0" pickDirection="0" doubleStops="0" palmMutes="0" harmonics="0" pinchHarmonics="0" hopo="0" tremolo="0" slides="0" unpitchedSlides="0" bends="0" tapping="0" vibrato="0" fretHandMutes="0" slapPop="0" twoFingerPicking="0" fiveFretChords="0" chordNotes="0" octaves="0" susChords="0" threeFingerChords="0" rhythmSide="0" solo="0" pathLead="1" pathRhythm="0" pathBass="0" routingRules="0" />
  <tonebase>Tone_Base</tonebase>
  <tonea></tonea>
  <toneb></toneb>
  <tonec></tonec>
  <toned></toned>
  <tones count="0" />
  <ebeats count="2">
    <ebeat time="0.500" measure="0" />
    <ebeat time="1.000" measure="-1" />
  </ebeats>
  <phrases count="1">
    <phrase maxDifficulty="0" name="COUNT" disparity="0" ignore="0" solo="0" />
  </phrases>
  <phraseIterations count="1">
    <phraseIteration time="0.500" phraseId="0" variation="" />
  </phraseIterations>
  <linkedDiffs count="0" />
  <phraseProperties count="0" />
  <chordTemplates count="0" />
  <fretHandMuteTemplates count="0" />
  <controls count="0" />
  <events count="0" />
  <sections count="1">
    <section name="chorus" number="1" startTime="0.500" endTime="120.000" />
  </sections>
  <levels count="1">
    <level difficulty="0">
      <anchors count="1">
        <anchor time="0.500" endTime="120.000" fret="1" width="4" />
      </anchors>
      <handShapes count="0" />
      <notes count="2">
        <note time="0.500" sustain="0.000" string="0" fret="5" leftHand="-1" slideTo="-1" slideUnpitchTo="-1" tap="0" pickDirection="0" slap="-1" pluck="-1" vibrato="0" maxBend="0" linkNext="0" accent="0" hammerOn="0" harmonic="0" ignore="0" fretHandMute="0" palmMute="0" pullOff="0" tremolo="0" pinchHarmonic="0" pickDirection="0" slap="0" pluck="0" rightHand="0" />
        <note time="1.000" sustain="0.500" string="1" fret="7" leftHand="-1" slideTo="-1" slideUnpitchTo="-1" tap="0" pickDirection="0" slap="-1" pluck="-1" vibrato="0" maxBend="0" linkNext="0" accent="0" hammerOn="1" harmonic="0" ignore="0" fretHandMute="0" palmMute="0" pullOff="0" tremolo="0" pinchHarmonic="0" rightHand="0">
          <bendValues count="1">
            <bendValue time="1.000" step="1" />
          </bendValues>
        </note>
      </notes>
      <chords count="0" />
    </level>
  </levels>
</song>"#;

    #[test]
    fn test_parse_basic() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        assert_eq!(arr.meta.song_name, "Test Song");
        assert_eq!(arr.meta.arrangement, "Lead");
        assert_eq!(arr.meta.artist_name, "Test Artist");
        assert_eq!(arr.meta.song_length, 120_000);
        assert_eq!(arr.meta.average_tempo, 120.0);
    }

    #[test]
    fn test_parse_ebeats() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        assert_eq!(arr.ebeats.len(), 2);
        assert_eq!(arr.ebeats[0].time, 500);
        assert_eq!(arr.ebeats[0].measure, 0);
        assert_eq!(arr.ebeats[1].time, 1000);
        assert_eq!(arr.ebeats[1].measure, -1);
    }

    #[test]
    fn test_parse_phrases() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        assert_eq!(arr.phrases.len(), 1);
        assert_eq!(arr.phrases[0].name, "COUNT");
        assert_eq!(arr.phrases[0].max_difficulty, 0);
    }

    #[test]
    fn test_parse_notes() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        assert_eq!(arr.levels.len(), 1);
        let level = &arr.levels[0];
        assert_eq!(level.notes.len(), 2);
        assert_eq!(level.notes[0].fret, 5);
        assert_eq!(level.notes[0].string, 0);
        assert_eq!(level.notes[1].fret, 7);
        assert!(level.notes[1].mask.contains(NoteMask::HAMMER_ON));
        assert_eq!(level.notes[1].bend_values.len(), 1);
        assert_eq!(level.notes[1].bend_values[0].step, 1.0);
    }

    #[test]
    fn test_parse_sections() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        assert_eq!(arr.sections.len(), 1);
        assert_eq!(arr.sections[0].name, "chorus");
        assert_eq!(arr.sections[0].start_time, 500);
    }

    #[test]
    fn test_roundtrip() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        let xml_out = arr.to_xml().unwrap();
        let arr2 = InstrumentalArrangement::from_xml(&xml_out).unwrap();
        assert_eq!(arr.meta.song_name, arr2.meta.song_name);
        assert_eq!(arr.ebeats.len(), arr2.ebeats.len());
        assert_eq!(arr.levels[0].notes.len(), arr2.levels[0].notes.len());
        assert_eq!(arr.levels[0].notes[0].fret, arr2.levels[0].notes[0].fret);
        assert_eq!(arr.levels[0].notes[1].bend_values.len(), arr2.levels[0].notes[1].bend_values.len());
    }

    #[test]
    fn test_empty_arrangement_roundtrip() {
        let arr = InstrumentalArrangement::default();
        let xml = arr.to_xml().unwrap();
        let arr2 = InstrumentalArrangement::from_xml(&xml).unwrap();
        assert_eq!(arr2.ebeats.len(), 0);
        assert_eq!(arr2.levels.len(), 0);
    }

    #[test]
    fn test_time_conversion() {
        assert_eq!(time_from_str("1.500"), 1500);
        assert_eq!(time_from_str("0.000"), 0);
        assert_eq!(time_to_str(1500), "1.500");
        assert_eq!(time_to_str(0), "0.000");
    }

    #[test]
    fn test_note_mask() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        let note = &arr.levels[0].notes[1];
        assert!(note.mask.contains(NoteMask::HAMMER_ON));
        assert!(!note.mask.contains(NoteMask::PULL_OFF));
    }
}
