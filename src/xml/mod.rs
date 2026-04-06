pub mod show_light;
pub mod utils;
pub mod vocal;

use std::{ffi::CString, path::Path};

use quick_xml::{events::Event, Reader};

use crate::error::{Error, Result};

pub use show_light::{ShowLight, ShowLights, FOG_MIN, FOG_MAX, BEAM_MIN, BEAM_MAX, BEAM_OFF};
pub use vocal::{XmlVocal, VocalsArrangement};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Tuning offsets for each string (semitones relative to E standard).
#[derive(Debug, Clone, Default)]
pub struct Tuning {
    pub string0: i16,
    pub string1: i16,
    pub string2: i16,
    pub string3: i16,
    pub string4: i16,
    pub string5: i16,
}

/// A minimal representation of a Rocksmith 2014 instrumental arrangement.
#[derive(Debug, Clone, Default)]
pub struct InstrumentalArrangement {
    pub title: String,
    pub arrangement: String,
    pub part: i32,
    pub offset: f32,
    pub cent_offset: f64,
    pub song_length: f32,
    pub average_tempo: f32,
    pub capo: i32,
    pub artist_name: String,
    pub artist_name_sort: String,
    pub album_name: String,
    pub album_year: i32,
    pub tuning: Tuning,
    pub last_conversion_date_time: String,
    /// Total number of `<note>` and `<chord>` elements across all levels.
    pub note_count: i32,
    /// Number of `<level>` elements.
    pub level_count: i32,

    // CString caches for FFI
    pub(crate) title_c: Option<CString>,
    pub(crate) arrangement_c: Option<CString>,
    pub(crate) artist_name_c: Option<CString>,
    pub(crate) artist_name_sort_c: Option<CString>,
    pub(crate) album_name_c: Option<CString>,
    pub(crate) last_conversion_date_time_c: Option<CString>,
}

impl InstrumentalArrangement {
    fn rebuild_cstrings(&mut self) {
        self.title_c = CString::new(self.title.as_bytes()).ok();
        self.arrangement_c = CString::new(self.arrangement.as_bytes()).ok();
        self.artist_name_c = CString::new(self.artist_name.as_bytes()).ok();
        self.artist_name_sort_c = CString::new(self.artist_name_sort.as_bytes()).ok();
        self.album_name_c = CString::new(self.album_name.as_bytes()).ok();
        self.last_conversion_date_time_c =
            CString::new(self.last_conversion_date_time.as_bytes()).ok();
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

impl InstrumentalArrangement {
    /// Parse a Rocksmith 2014 arrangement XML from `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_str(&text)
    }

    /// Parse from a string slice.
    pub fn from_str(xml: &str) -> Result<Self> {
        let mut arr = InstrumentalArrangement::default();
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_tag = String::new();
        let mut note_count = 0i32;
        let mut level_count = 0i32;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let tag = std::str::from_utf8(e.name().as_ref())
                        .unwrap_or("")
                        .to_owned();

                    match tag.as_str() {
                        "tuning" => {
                            for attr in e.attributes().flatten() {
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .unwrap_or("")
                                    .to_owned();
                                let val: i16 = std::str::from_utf8(&attr.value)
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0);
                                match key.as_str() {
                                    "string0" => arr.tuning.string0 = val,
                                    "string1" => arr.tuning.string1 = val,
                                    "string2" => arr.tuning.string2 = val,
                                    "string3" => arr.tuning.string3 = val,
                                    "string4" => arr.tuning.string4 = val,
                                    "string5" => arr.tuning.string5 = val,
                                    _ => {}
                                }
                            }
                        }
                        "note" | "chord" => {
                            note_count += 1;
                        }
                        "level" => {
                            level_count += 1;
                        }
                        _ => {}
                    }

                    current_tag = tag;
                }
                Ok(Event::Text(e)) => {
                    let text = e
                        .unescape()
                        .map(|s| s.into_owned())
                        .unwrap_or_default();
                    match current_tag.as_str() {
                        "title" => arr.title = text,
                        "arrangement" => arr.arrangement = text,
                        "part" => arr.part = text.parse().unwrap_or(0),
                        "offset" => arr.offset = text.parse().unwrap_or(0.0),
                        "centOffset" => arr.cent_offset = text.parse().unwrap_or(0.0),
                        "songLength" => arr.song_length = text.parse().unwrap_or(0.0),
                        "averageTempo" => arr.average_tempo = text.parse().unwrap_or(0.0),
                        "capo" => arr.capo = text.parse().unwrap_or(0),
                        "artistName" => arr.artist_name = text,
                        "artistNameSort" => arr.artist_name_sort = text,
                        "albumName" => arr.album_name = text,
                        "albumYear" => arr.album_year = text.parse().unwrap_or(0),
                        "lastConversionDateTime" => arr.last_conversion_date_time = text,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlParse(format!("{e}"))),
                _ => {}
            }
            buf.clear();
        }

        arr.note_count = note_count;
        arr.level_count = level_count;
        arr.rebuild_cstrings();
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_XML: &str = r#"<?xml version='1.0' encoding='utf-8'?>
<song>
  <title>Test Song</title>
  <arrangement>Lead</arrangement>
  <part>1</part>
  <offset>0</offset>
  <centOffset>0</centOffset>
  <songLength>10.000</songLength>
  <averageTempo>120.000</averageTempo>
  <tuning string0="0" string1="0" string2="0" string3="0" string4="0" string5="0" />
  <capo>0</capo>
  <artistName>Test Artist</artistName>
  <artistNameSort>Test Artist Sort</artistNameSort>
  <albumName>Test Album</albumName>
  <albumYear>2024</albumYear>
  <lastConversionDateTime>2024-01-01 00:00:00</lastConversionDateTime>
  <levels count="1">
    <level difficulty="0">
      <notes count="2">
        <note time="1.000" string="0" fret="5" />
        <note time="2.000" string="1" fret="7" />
      </notes>
    </level>
  </levels>
</song>"#;

    #[test]
    fn parse_minimal_xml() {
        let arr = InstrumentalArrangement::from_str(MINIMAL_XML).unwrap();
        assert_eq!(arr.title, "Test Song");
        assert_eq!(arr.arrangement, "Lead");
        assert_eq!(arr.part, 1);
        assert!((arr.song_length - 10.0).abs() < 0.001);
        assert!((arr.average_tempo - 120.0).abs() < 0.001);
        assert_eq!(arr.artist_name, "Test Artist");
        assert_eq!(arr.artist_name_sort, "Test Artist Sort");
        assert_eq!(arr.album_name, "Test Album");
        assert_eq!(arr.album_year, 2024);
        assert_eq!(arr.note_count, 2);
        assert_eq!(arr.level_count, 1);
        assert_eq!(arr.tuning.string0, 0);
    }

    #[test]
    fn parse_drop_d_tuning() {
        let xml = r#"<song>
            <title>Drop D</title>
            <arrangement>Lead</arrangement>
            <songLength>5.0</songLength>
            <averageTempo>100.0</averageTempo>
            <artistName>A</artistName>
            <albumName>B</albumName>
            <tuning string0="-2" string1="0" string2="0"
                    string3="0"  string4="0" string5="0" />
        </song>"#;
        let arr = InstrumentalArrangement::from_str(xml).unwrap();
        assert_eq!(arr.tuning.string0, -2);
        assert_eq!(arr.tuning.string1, 0);
    }
}
