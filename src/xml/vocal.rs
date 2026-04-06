//! XML-format Vocal type mirroring `Rocksmith2014.XML.Vocal` from Rocksmith2014.NET v3.5.0.
//!
//! Note: this is the **XML** vocal (time stored as milliseconds, lyric as string),
//! distinct from the SNG binary vocal in `crate::sng::types`.

use std::path::Path;

use quick_xml::{events::Event, Reader};

use crate::error::{Error, Result};
use super::utils::time_code_from_float_string;

// ---------------------------------------------------------------------------
// XmlVocal
// ---------------------------------------------------------------------------

/// A single vocal event from a `Vocals.xml` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlVocal {
    /// Time in milliseconds.
    pub time: i32,
    /// MIDI note (not used in RS2014; default 60).
    pub note: u8,
    /// Length in milliseconds.
    pub length: i32,
    /// Lyric string (may end with `+` to mark phrase continuation).
    pub lyric: String,
}

impl XmlVocal {
    /// Create a new vocal.
    pub fn new(time: i32, length: i32, lyric: impl Into<String>, note: u8) -> Self {
        Self { time, note, length, lyric: lyric.into() }
    }
}

// ---------------------------------------------------------------------------
// VocalsArrangement
// ---------------------------------------------------------------------------

/// A parsed `Vocals.xml` arrangement.
#[derive(Debug, Default)]
pub struct VocalsArrangement {
    /// The list of vocals in the arrangement.
    pub vocals: Vec<XmlVocal>,
}

impl VocalsArrangement {
    /// Load a `Vocals.xml` file from `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_str(&text)
    }

    /// Parse from an XML string.
    pub fn from_str(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut vocals = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"vocal" => {
                    let mut time = 0i32;
                    let mut note = 60u8;
                    let mut length = 0i32;
                    let mut lyric = String::new();

                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        match key {
                            "time" => {
                                let v = std::str::from_utf8(&attr.value).unwrap_or("0");
                                time = time_code_from_float_string(v);
                            }
                            "note" => {
                                note = std::str::from_utf8(&attr.value)
                                    .unwrap_or("60")
                                    .parse()
                                    .unwrap_or(60);
                            }
                            "length" => {
                                let v = std::str::from_utf8(&attr.value).unwrap_or("0");
                                length = time_code_from_float_string(v);
                            }
                            "lyric" => {
                                lyric = std::str::from_utf8(&attr.value)
                                    .unwrap_or("")
                                    .to_owned();
                            }
                            _ => {}
                        }
                    }
                    vocals.push(XmlVocal { time, note, length, lyric });
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlParse(format!("{e}"))),
                _ => {}
            }
            buf.clear();
        }

        Ok(VocalsArrangement { vocals })
    }

    /// Serialise this vocals arrangement to an XML string.
    pub fn to_xml_string(&self) -> String {
        use super::utils::time_code_to_string;
        let mut xml = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<vocals count=\"{}\">\n",
            self.vocals.len()
        );
        for v in &self.vocals {
            xml.push_str(&format!(
                "  <vocal time=\"{}\" note=\"{}\" length=\"{}\" lyric=\"{}\" />\n",
                time_code_to_string(v.time),
                v.note,
                time_code_to_string(v.length),
                v.lyric,
            ));
        }
        xml.push_str("</vocals>\n");
        xml
    }

    /// Save this vocals arrangement to an XML file at `path`.
    ///
    /// Mirrors `Vocals.Save()` from Rocksmith2014.NET.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.to_xml_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------ //
    // CopyConstructorCopiesAllValues  (mirrors VocalTests)
    // ------------------------------------------------------------------ //
    #[test]
    fn copy_constructor_copies_all_values() {
        let v1 = XmlVocal::new(5_000, 1_000, "Hello+", 50);
        let mut v2 = v1.clone();

        assert_eq!(v2.time,   5_000);
        assert_eq!(v2.note,   50);
        assert_eq!(v2.length, 1_000);
        assert_eq!(v2.lyric,  "Hello+");

        // Modifying v2.lyric must not affect v1 (clone is deep in Rust)
        v2.lyric = "Modified".into();
        assert_ne!(v2.lyric, v1.lyric);
    }

    // ------------------------------------------------------------------ //
    // CanBeLoadedFromAnXmlFile  (mirrors VocalTests — uses Vocals.xml data inline)
    // ------------------------------------------------------------------ //
    const VOCALS_XML: &str = r#"<?xml version='1.0' encoding='UTF-8'?>
<vocals count="8">
  <vocal time="25.330" note="254" length="0.600" lyric="Test"/>
  <vocal time="26.530" note="254" length="0.600" lyric="lyrics+"/>
  <vocal time="27.730" note="254" length="0.525" lyric="Lo-"/>
  <vocal time="28.330" note="254" length="0.225" lyric="rem"/>
  <vocal time="28.630" note="254" length="0.075" lyric="ip-"/>
  <vocal time="28.780" note="254" length="0.600" lyric="sum+"/>
  <vocal time="30.130" note="254" length="0.600" lyric="Dolor"/>
  <vocal time="31.330" note="254" length="0.600" lyric="sit+"/>
</vocals>"#;

    #[test]
    fn can_be_loaded_from_xml() {
        let arr = VocalsArrangement::from_str(VOCALS_XML).unwrap();
        assert_eq!(arr.vocals.len(), 8);
        assert_eq!(arr.vocals[0].lyric, "Test");
        assert_eq!(arr.vocals[7].lyric, "sit+");
    }

    #[test]
    fn vocal_times_are_parsed_as_ms() {
        let arr = VocalsArrangement::from_str(VOCALS_XML).unwrap();
        // time="25.330" → 25330 ms
        assert_eq!(arr.vocals[0].time, 25_330);
        // length="0.600" → 600 ms
        assert_eq!(arr.vocals[0].length, 600);
    }
}
