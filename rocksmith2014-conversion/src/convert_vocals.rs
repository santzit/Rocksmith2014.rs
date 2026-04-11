//! Vocal conversion between XML and SNG formats.
//!
//! Mirrors `ConvertVocals.fs` in Rocksmith2014.NET.

use rocksmith2014_sng::{Sng, SymbolDefinition, SymbolsHeader, SymbolsTexture};
use rocksmith2014_xml::{GlyphDefinition, GlyphDefinitions, Vocal as XmlVocal};

use crate::utils::{bytes_to_string, ms_to_sec, sec_to_ms, string_to_bytes};

/// Font selection for vocal SNG conversion.
///
/// Mirrors `FontOption` in the .NET reference implementation.
pub enum FontOption<'a> {
    /// Use the built-in default Rocksmith lyrics font (192 symbols).
    DefaultFont,
    /// Use a custom font atlas with the given glyph definitions and asset path.
    CustomFont(&'a GlyphDefinitions, &'a str),
}

/// Default symbols — loaded lazily from the embedded `default_symbols.bin`.
static DEFAULT_SYMBOLS_BIN: &[u8] = include_bytes!("../default_symbols.bin");

/// Parses the embedded `default_symbols.bin` into a Vec of SymbolDefinition.
fn default_symbols() -> Vec<SymbolDefinition> {
    // Each SymbolDefinition on disk: 12-byte symbol + 4×f32 outer + 4×f32 inner = 44 bytes
    const RECORD: usize = 44;
    let count = DEFAULT_SYMBOLS_BIN.len() / RECORD;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let base = i * RECORD;
        let mut symbol = [0u8; 12];
        symbol.copy_from_slice(&DEFAULT_SYMBOLS_BIN[base..base + 12]);
        let f = |off: usize| {
            f32::from_le_bytes(
                DEFAULT_SYMBOLS_BIN[base + off..base + off + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        let outer = rocksmith2014_sng::Rect {
            ymin: f(12),
            xmin: f(16),
            ymax: f(20),
            xmax: f(24),
        };
        let inner = rocksmith2014_sng::Rect {
            ymin: f(28),
            xmin: f(32),
            ymax: f(36),
            xmax: f(40),
        };
        out.push(SymbolDefinition {
            symbol,
            outer,
            inner,
        });
    }
    out
}

/// Default symbols texture used when the default font is selected.
fn default_texture() -> SymbolsTexture {
    let path = r"assets\ui\lyrics\lyrics.dds";
    let mut font = [0u8; 128];
    let b = path.as_bytes();
    font[..b.len()].copy_from_slice(b);
    SymbolsTexture {
        font,
        font_path_length: path.len() as i32,
        width: 1024,
        height: 512,
    }
}

/// Default symbol headers used when the default font is selected.
fn default_headers() -> Vec<SymbolsHeader> {
    vec![
        SymbolsHeader::default(),
        SymbolsHeader {
            id: 1,
            ..Default::default()
        },
    ]
}

/// Converts an XML `GlyphDefinition` to an SNG `SymbolDefinition`.
///
/// Mirrors `XmlToSng.convertSymbolDefinition` in the .NET reference.
pub fn convert_symbol_definition(g: &GlyphDefinition) -> SymbolDefinition {
    SymbolDefinition {
        symbol: string_to_bytes::<12>(&g.symbol),
        outer: rocksmith2014_sng::Rect {
            xmin: g.outer_x_min,
            xmax: g.outer_x_max,
            ymin: g.outer_y_min,
            ymax: g.outer_y_max,
        },
        inner: rocksmith2014_sng::Rect {
            xmin: g.inner_x_min,
            xmax: g.inner_x_max,
            ymin: g.inner_y_min,
            ymax: g.inner_y_max,
        },
    }
}

/// Converts an SNG `SymbolDefinition` back to an XML `GlyphDefinition`.
///
/// Mirrors `SngToXml.convertSymbolDefinition` in the .NET reference.
pub fn convert_symbol_definition_to_xml(s: &SymbolDefinition) -> GlyphDefinition {
    GlyphDefinition {
        symbol: bytes_to_string(&s.symbol),
        inner_y_min: s.inner.ymin,
        inner_y_max: s.inner.ymax,
        inner_x_min: s.inner.xmin,
        inner_x_max: s.inner.xmax,
        outer_y_min: s.outer.ymin,
        outer_y_max: s.outer.ymax,
        outer_x_min: s.outer.xmin,
        outer_x_max: s.outer.xmax,
    }
}

/// Converts a list of XML vocals into an SNG `Sng` containing only vocal data.
///
/// Mirrors `ConvertVocals.xmlToSng` in the .NET reference implementation.
pub fn xml_vocals_to_sng(font: FontOption<'_>, xml_vocals: &[XmlVocal]) -> Sng {
    let vocals: Vec<rocksmith2014_sng::Vocal> = xml_vocals.iter().map(convert_vocal).collect();

    let (headers, textures, symbols) = match font {
        FontOption::DefaultFont => (
            default_headers(),
            vec![default_texture()],
            default_symbols(),
        ),
        FontOption::CustomFont(glyphs, asset_path) => {
            let mut font_bytes = [0u8; 128];
            let b = asset_path.as_bytes();
            let len = b.len().min(128);
            font_bytes[..len].copy_from_slice(&b[..len]);
            let texture = SymbolsTexture {
                font: font_bytes,
                font_path_length: asset_path.len().min(128) as i32,
                width: glyphs.texture_width,
                height: glyphs.texture_height,
            };
            let syms = glyphs
                .glyphs
                .iter()
                .map(convert_symbol_definition)
                .collect();
            (vec![SymbolsHeader::default()], vec![texture], syms)
        }
    };

    Sng {
        vocals,
        symbols_headers: headers,
        symbols_textures: textures,
        symbol_definitions: symbols,
        ..Default::default()
    }
}

/// Converts the vocals in an SNG back to a list of XML vocals.
///
/// Mirrors `ConvertVocals.sngToXml` in the .NET reference implementation.
pub fn sng_vocals_to_xml(sng: &Sng) -> Vec<XmlVocal> {
    sng.vocals.iter().map(convert_vocal_to_xml).collect()
}

/// Extracts glyph data from an SNG into a `GlyphDefinitions` struct.
///
/// Mirrors `ConvertVocals.extractGlyphData` in the .NET reference implementation.
pub fn extract_glyph_data(sng: &Sng) -> GlyphDefinitions {
    let glyphs = sng
        .symbol_definitions
        .iter()
        .map(convert_symbol_definition_to_xml)
        .collect();
    let (w, h) = if let Some(t) = sng.symbols_textures.first() {
        (t.width, t.height)
    } else {
        (0, 0)
    };
    GlyphDefinitions {
        texture_width: w,
        texture_height: h,
        glyphs,
    }
}

/// Converts a single XML vocal entry to an SNG vocal entry.
pub fn xml_convert_vocal(v: &XmlVocal) -> rocksmith2014_sng::Vocal {
    convert_vocal(v)
}

/// Converts a single SNG vocal entry to an XML vocal entry.
pub fn sng_convert_vocal(v: &rocksmith2014_sng::Vocal) -> XmlVocal {
    convert_vocal_to_xml(v)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Converts an XML `Vocal` to an SNG `Vocal`.
fn convert_vocal(v: &XmlVocal) -> rocksmith2014_sng::Vocal {
    // Encode lyric string into the 48-byte fixed-size field (null-terminated, truncated).
    let encoded = v.lyric.as_bytes();
    // Reserve 1 byte for null terminator; truncate at 47 bytes
    let max = 47.min(encoded.len());
    let mut lyric = [0u8; 48];
    lyric[..max].copy_from_slice(&encoded[..max]);
    rocksmith2014_sng::Vocal {
        time: ms_to_sec(v.time),
        note: v.note as i32,
        length: ms_to_sec(v.length),
        lyric,
    }
}

/// Converts an SNG `Vocal` back to an XML `Vocal`.
fn convert_vocal_to_xml(v: &rocksmith2014_sng::Vocal) -> XmlVocal {
    XmlVocal {
        time: sec_to_ms(v.time),
        note: v.note as u8,
        length: sec_to_ms(v.length),
        lyric: bytes_to_string(&v.lyric),
    }
}
