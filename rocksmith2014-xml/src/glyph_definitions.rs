//! GlyphDefinitions and GlyphDefinition XML types.
//!
//! Mirrors `GlyphDefinitions.cs` in Rocksmith2014.NET.

use crate::Result;
use quick_xml::events::{BytesDecl, BytesStart, Event as XmlEvent};
use quick_xml::{Reader, Writer};
use std::fs;
use std::io::BufWriter;
use std::path::Path;

/// Represents a single glyph (symbol) definition used in a custom lyrics font.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlyphDefinition {
    pub symbol: String,
    pub inner_y_min: f32,
    pub inner_y_max: f32,
    pub inner_x_min: f32,
    pub inner_x_max: f32,
    pub outer_y_min: f32,
    pub outer_y_max: f32,
    pub outer_x_min: f32,
    pub outer_x_max: f32,
}

/// Holds all glyph definitions for a custom lyrics texture atlas.
///
/// Mirrors `GlyphDefinitions` in the .NET reference implementation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlyphDefinitions {
    pub texture_width: i32,
    pub texture_height: i32,
    pub glyphs: Vec<GlyphDefinition>,
}

impl GlyphDefinitions {
    /// Loads glyph definitions from a `.glyphs.xml` file.
    ///
    /// Mirrors `GlyphDefinitions.Load` in the .NET implementation.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let xml = fs::read_to_string(path)?;
        Self::from_xml(&xml)
    }

    /// Removes characters that are illegal in XML 1.0 from a string.
    fn sanitize_xml(s: &str) -> String {
        s.chars()
            .filter(|&c| {
                matches!(c,
                    '\t' | '\n' | '\r'
                    | '\u{0020}'..='\u{D7FF}'
                    | '\u{E000}'..='\u{FFFD}'
                    | '\u{10000}'..='\u{10FFFF}'
                )
            })
            .collect()
    }

    /// Saves glyph definitions to a `.glyphs.xml` file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = fs::File::create(path)?;
        let buf = BufWriter::new(file);
        let mut w = Writer::new_with_indent(buf, b' ', 2);
        w.write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
        let mut root = BytesStart::new("GlyphDefinitions");
        root.push_attribute(("TextureWidth", self.texture_width.to_string().as_str()));
        root.push_attribute(("TextureHeight", self.texture_height.to_string().as_str()));
        w.write_event(XmlEvent::Start(root))?;
        for g in &self.glyphs {
            let safe_symbol = Self::sanitize_xml(&g.symbol);
            let mut elem = BytesStart::new("GlyphDefinition");
            elem.push_attribute(("Symbol", safe_symbol.as_str()));
            elem.push_attribute(("InnerYMin", g.inner_y_min.to_string().as_str()));
            elem.push_attribute(("InnerYMax", g.inner_y_max.to_string().as_str()));
            elem.push_attribute(("InnerXMin", g.inner_x_min.to_string().as_str()));
            elem.push_attribute(("InnerXMax", g.inner_x_max.to_string().as_str()));
            elem.push_attribute(("OuterYMin", g.outer_y_min.to_string().as_str()));
            elem.push_attribute(("OuterYMax", g.outer_y_max.to_string().as_str()));
            elem.push_attribute(("OuterXMin", g.outer_x_min.to_string().as_str()));
            elem.push_attribute(("OuterXMax", g.outer_x_max.to_string().as_str()));
            w.write_event(XmlEvent::Empty(elem))?;
        }
        w.write_event(XmlEvent::End(quick_xml::events::BytesEnd::new(
            "GlyphDefinitions",
        )))?;
        Ok(())
    }

    /// Parses glyph definitions from an XML string.
    pub fn from_xml(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut result = GlyphDefinitions::default();
        loop {
            match reader.read_event()? {
                XmlEvent::Start(e) | XmlEvent::Empty(e)
                    if e.name().as_ref() == b"GlyphDefinitions" =>
                {
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        let val = std::str::from_utf8(attr.value.as_ref()).unwrap_or("");
                        match attr.key.as_ref() {
                            b"TextureWidth" => result.texture_width = val.parse().unwrap_or(0),
                            b"TextureHeight" => result.texture_height = val.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }
                XmlEvent::Empty(e) if e.name().as_ref() == b"GlyphDefinition" => {
                    let mut g = GlyphDefinition::default();
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        let val = std::str::from_utf8(attr.value.as_ref()).unwrap_or("");
                        match attr.key.as_ref() {
                            b"Symbol" => g.symbol = val.to_string(),
                            b"InnerYMin" => g.inner_y_min = val.parse().unwrap_or(0.0),
                            b"InnerYMax" => g.inner_y_max = val.parse().unwrap_or(0.0),
                            b"InnerXMin" => g.inner_x_min = val.parse().unwrap_or(0.0),
                            b"InnerXMax" => g.inner_x_max = val.parse().unwrap_or(0.0),
                            b"OuterYMin" => g.outer_y_min = val.parse().unwrap_or(0.0),
                            b"OuterYMax" => g.outer_y_max = val.parse().unwrap_or(0.0),
                            b"OuterXMin" => g.outer_x_min = val.parse().unwrap_or(0.0),
                            b"OuterXMax" => g.outer_x_max = val.parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
                    result.glyphs.push(g);
                }
                XmlEvent::Eof => break,
                _ => {}
            }
        }
        Ok(result)
    }
}
