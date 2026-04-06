//! ShowLight type mirroring `Rocksmith2014.XML.ShowLight` from Rocksmith2014.NET v3.5.0.

use std::path::Path;

use quick_xml::{events::Event, Reader};

use crate::error::{Error, Result};
use super::utils::{time_code_from_float_string, time_code_to_string};

// ---------------------------------------------------------------------------
// Constants — exact values from ShowLight.cs
// ---------------------------------------------------------------------------

/// Minimum fog note (inclusive).
pub const FOG_MIN: u8 = 24;
/// Maximum fog note (inclusive).
pub const FOG_MAX: u8 = 35;
/// Beam-off note.
pub const BEAM_OFF: u8 = 42;
/// Minimum beam note (inclusive).
pub const BEAM_MIN: u8 = 48;
/// Maximum beam note (inclusive).
pub const BEAM_MAX: u8 = 59;
/// Lasers-off note.
pub const LASERS_OFF: u8 = 66;
/// Lasers-on note.
pub const LASERS_ON: u8 = 67;

// ---------------------------------------------------------------------------
// ShowLight
// ---------------------------------------------------------------------------

/// A single Rocksmith 2014 show-light event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowLight {
    /// Time in milliseconds.
    pub time: i32,
    /// Show-light note value.
    pub note: u8,
}

impl ShowLight {
    /// Returns `true` if this is a fog note (`FOG_MIN..=FOG_MAX`).
    #[inline]
    pub fn is_fog(&self) -> bool {
        self.note >= FOG_MIN && self.note <= FOG_MAX
    }

    /// Returns `true` if this is a beam note (`BEAM_MIN..=BEAM_MAX` or `BEAM_OFF`).
    #[inline]
    pub fn is_beam(&self) -> bool {
        (self.note >= BEAM_MIN && self.note <= BEAM_MAX) || self.note == BEAM_OFF
    }

    /// Returns the formatted time string, e.g. `"18.731"`.
    pub fn time_string(&self) -> String {
        time_code_to_string(self.time)
    }
}

// ---------------------------------------------------------------------------
// ShowLights collection
// ---------------------------------------------------------------------------

/// A parsed `Showlights.xml` file.
#[derive(Debug, Default)]
pub struct ShowLights(pub Vec<ShowLight>);

impl ShowLights {
    /// Load a `showlights` XML file from `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_str(&text)
    }

    /// Parse from an XML string.
    pub fn from_str(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut lights = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"showlight" => {
                    let mut time = 0i32;
                    let mut note = 0u8;
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        match key {
                            "time" => {
                                let v = std::str::from_utf8(&attr.value).unwrap_or("0");
                                time = time_code_from_float_string(v);
                            }
                            "note" => {
                                note = std::str::from_utf8(&attr.value)
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0);
                            }
                            _ => {}
                        }
                    }
                    lights.push(ShowLight { time, note });
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::XmlParse(format!("{e}"))),
                _ => {}
            }
            buf.clear();
        }

        Ok(ShowLights(lights))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------ //
    // Fog range  (mirrors ShowLightTests.FogRangeTest)
    // ------------------------------------------------------------------ //
    #[test]
    fn fog_range_test() {
        let mut sl = ShowLight { time: 0, note: 0 };

        for note in FOG_MIN..=FOG_MAX {
            sl.note = note;
            assert!(sl.is_fog(), "note {note} should be fog");
        }

        sl.note = FOG_MAX + 1;
        assert!(!sl.is_fog(), "note {} should NOT be fog", FOG_MAX + 1);

        sl.note = FOG_MIN - 1;
        assert!(!sl.is_fog(), "note {} should NOT be fog", FOG_MIN - 1);
    }

    // ------------------------------------------------------------------ //
    // Beam range  (mirrors ShowLightTests.BeamRangeTest)
    // ------------------------------------------------------------------ //
    #[test]
    fn beam_range_test() {
        let mut sl = ShowLight { time: 50, note: FOG_MIN };

        for note in BEAM_MIN..=BEAM_MAX {
            sl.note = note;
            assert!(sl.is_beam(), "note {note} should be beam");
        }

        sl.note = BEAM_OFF;
        assert!(sl.is_beam(), "BEAM_OFF ({BEAM_OFF}) should be beam");

        sl.note = BEAM_MAX + 1;
        assert!(!sl.is_beam(), "note {} should NOT be beam", BEAM_MAX + 1);

        sl.note = BEAM_MIN - 1;
        assert!(!sl.is_beam(), "note {} should NOT be beam", BEAM_MIN - 1);
    }

    // ------------------------------------------------------------------ //
    // Note: the "ListOfShowlightsCanBeSavedToXmlFile" test involves writing
    // XML, which we do not implement yet.  It is covered in xml_types_tests.rs
    // via the "save" stub assertion.
    // ------------------------------------------------------------------ //
}
