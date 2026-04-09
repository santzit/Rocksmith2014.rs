//! ShowLight type and ShowLights XML serialization.
//!
//! Mirrors `ShowLight.cs` and `ShowLights.cs` in Rocksmith2014.NET.

use crate::parser::{time_from_str, time_to_str};
use crate::Result;
use quick_xml::events::{BytesDecl, BytesStart, Event as XmlEvent};
use quick_xml::{Reader, Writer};
use std::fs;
use std::io::BufWriter;
use std::path::Path;

/// Represents a show light entry from a Showlights.xml file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShowLight {
    /// The time code of the show light (milliseconds).
    pub time: i32,
    /// The note of the show light.
    ///
    /// Valid values: 24-35 (fog), 42 (beam off), 48-59 (beam), 66-67 (lasers).
    pub note: u8,
}

impl ShowLight {
    /// Minimum fog note value.
    pub const FOG_MIN: u8 = 24;
    /// Maximum fog note value.
    pub const FOG_MAX: u8 = 35;
    /// Beam off note value.
    pub const BEAM_OFF: u8 = 42;
    /// Minimum beam note value.
    pub const BEAM_MIN: u8 = 48;
    /// Maximum beam note value.
    pub const BEAM_MAX: u8 = 59;
    /// Lasers off note value.
    pub const LASERS_OFF: u8 = 66;
    /// Lasers on note value.
    pub const LASERS_ON: u8 = 67;

    /// Creates a new show light with the given time and note.
    pub fn new(time: i32, note: u8) -> Self {
        Self { time, note }
    }

    /// Returns true if this is a beam note (BeamMin..=BeamMax or BeamOff).
    pub fn is_beam(&self) -> bool {
        (self.note >= Self::BEAM_MIN && self.note <= Self::BEAM_MAX) || self.note == Self::BEAM_OFF
    }

    /// Returns true if this is a fog note (FogMin..=FogMax).
    pub fn is_fog(&self) -> bool {
        self.note >= Self::FOG_MIN && self.note <= Self::FOG_MAX
    }
}

/// Saves a list of show lights to an XML file.
///
/// Mirrors `ShowLights.Save` in the .NET reference implementation.
pub fn save(file_name: impl AsRef<Path>, show_lights: &[ShowLight]) -> Result<()> {
    let file = fs::File::create(file_name)?;
    let buf = BufWriter::new(file);
    let mut w = Writer::new_with_indent(buf, b' ', 2);
    w.write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut root = BytesStart::new("showlights");
    root.push_attribute(("count", show_lights.len().to_string().as_str()));
    w.write_event(XmlEvent::Start(root))?;
    for sl in show_lights {
        let mut elem = BytesStart::new("showlight");
        elem.push_attribute(("time", time_to_str(sl.time).as_str()));
        elem.push_attribute(("note", sl.note.to_string().as_str()));
        w.write_event(XmlEvent::Empty(elem))?;
    }
    w.write_event(XmlEvent::End(quick_xml::events::BytesEnd::new(
        "showlights",
    )))?;
    Ok(())
}

/// Loads a list of show lights from an XML file.
///
/// Mirrors `ShowLights.Load` in the .NET reference implementation.
pub fn load(file_name: impl AsRef<Path>) -> Result<Vec<ShowLight>> {
    let xml = fs::read_to_string(file_name)?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut show_lights = Vec::new();
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"showlight" => {
                let mut sl = ShowLight::default();
                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let val = std::str::from_utf8(attr.value.as_ref()).unwrap_or("");
                    match attr.key.as_ref() {
                        b"time" => sl.time = time_from_str(val),
                        b"note" => sl.note = val.parse().unwrap_or(0),
                        _ => {}
                    }
                }
                show_lights.push(sl);
            }
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(show_lights)
}
