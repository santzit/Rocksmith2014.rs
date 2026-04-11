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

pub mod glyph_definitions;
mod parser;
pub mod show_light;
mod types;
pub mod utils;
pub mod vocal;
mod writer;

pub use glyph_definitions::{GlyphDefinition, GlyphDefinitions};
pub use show_light::ShowLight;
pub use types::{
    Anchor, ArrangementEvent, ArrangementProperties, BendValue, Chord, ChordMask, ChordNote,
    ChordTemplate, Ebeat, HandShape, HeroLevel, InstrumentalArrangement, Level, LinkedDiff,
    MetaData, NewLinkedDiff, Note, NoteMask, Phrase, PhraseIteration, PhraseProperty, Section,
    ToneChange, Tuning,
};
pub use vocal::Vocal;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event as XmlEvent};
use quick_xml::{Reader, Writer};
use std::fs;
use std::path::Path;

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

impl InstrumentalArrangement {
    pub fn from_xml(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        loop {
            match reader.read_event()? {
                XmlEvent::Start(e) if e.name().as_ref() == b"song" => {
                    return parser::parse_song(&mut reader);
                }
                XmlEvent::Eof => break,
                _ => {}
            }
        }
        Ok(InstrumentalArrangement::default())
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut w = Writer::new_with_indent(Vec::new(), b' ', 2);
        w.write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
        let song_start = BytesStart::new("song");
        w.write_event(XmlEvent::Start(song_start))?;
        writer::write_arrangement(self, &mut w)?;
        w.write_event(XmlEvent::End(BytesEnd::new("song")))?;
        Ok(String::from_utf8(w.into_inner())?)
    }

    /// Removes Dynamic Difficulty, keeping only the highest difficulty level.
    ///
    /// Mirrors `InstrumentalArrangement.RemoveDD` in Rocksmith2014.NET.
    pub fn remove_dd(&mut self) {
        if self.levels.len() > 1 {
            let max_diff = self.levels.iter().map(|l| l.difficulty).max().unwrap_or(0);
            self.levels.retain(|l| l.difficulty == max_diff);
        }
        if let Some(level) = self.levels.first_mut() {
            level.difficulty = 0;
        }
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
        assert_eq!(
            arr.levels[0].notes[1].bend_values.len(),
            arr2.levels[0].notes[1].bend_values.len()
        );
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
    fn test_note_mask() {
        let arr = InstrumentalArrangement::from_xml(SIMPLE_XML).unwrap();
        let note = &arr.levels[0].notes[1];
        assert!(note.mask.contains(NoteMask::HAMMER_ON));
        assert!(!note.mask.contains(NoteMask::PULL_OFF));
    }
}
