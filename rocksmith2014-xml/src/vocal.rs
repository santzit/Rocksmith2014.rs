//! Vocal type and Vocals XML serialization.
//!
//! Mirrors `Vocal.cs` and `Vocals.cs` in Rocksmith2014.NET.

use crate::parser::{time_from_str, time_to_str};
use crate::Result;
use quick_xml::events::{BytesDecl, BytesStart, Event as XmlEvent};
use quick_xml::{Reader, Writer};
use std::fs;
use std::io::BufWriter;
use std::path::Path;

/// Represents a vocal entry in a Vocals.xml file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Vocal {
    /// The time code of the vocal (milliseconds).
    pub time: i32,
    /// The MIDI note of the vocal. Not used in Rocksmith 2014.
    pub note: u8,
    /// The length of the vocal (milliseconds).
    pub length: i32,
    /// The lyric string.
    pub lyric: String,
}

impl Vocal {
    /// Creates a new vocal with the given properties.
    pub fn new(time: i32, length: i32, lyric: impl Into<String>, note: u8) -> Self {
        Self {
            time,
            note,
            length,
            lyric: lyric.into(),
        }
    }

    /// Creates a copy of another vocal.
    pub fn copy(other: &Self) -> Self {
        other.clone()
    }
}

/// Saves a list of vocals to an XML file.
///
/// Mirrors `Vocals.Save` in the .NET reference implementation.
pub fn save(file_name: impl AsRef<Path>, vocals: &[Vocal]) -> Result<()> {
    let file = fs::File::create(file_name)?;
    let buf = BufWriter::new(file);
    let mut w = Writer::new_with_indent(buf, b' ', 2);
    w.write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut root = BytesStart::new("vocals");
    root.push_attribute(("count", vocals.len().to_string().as_str()));
    w.write_event(XmlEvent::Start(root))?;
    for v in vocals {
        let mut elem = BytesStart::new("vocal");
        elem.push_attribute(("time", time_to_str(v.time).as_str()));
        elem.push_attribute(("note", v.note.to_string().as_str()));
        elem.push_attribute(("length", time_to_str(v.length).as_str()));
        elem.push_attribute(("lyric", v.lyric.as_str()));
        w.write_event(XmlEvent::Empty(elem))?;
    }
    w.write_event(XmlEvent::End(quick_xml::events::BytesEnd::new("vocals")))?;
    Ok(())
}

/// Loads a list of vocals from an XML file.
///
/// Mirrors `Vocals.Load` in the .NET reference implementation.
pub fn load(file_name: impl AsRef<Path>) -> Result<Vec<Vocal>> {
    let xml = fs::read_to_string(file_name)?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut vocals = Vec::new();
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"vocal" => {
                let mut v = Vocal::default();
                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let val = std::str::from_utf8(attr.value.as_ref()).unwrap_or("");
                    match attr.key.as_ref() {
                        b"time" => v.time = time_from_str(val),
                        b"note" => v.note = val.parse().unwrap_or(0),
                        b"length" => v.length = time_from_str(val),
                        b"lyric" => v.lyric = val.to_string(),
                        _ => {}
                    }
                }
                vocals.push(v);
            }
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(vocals)
}
