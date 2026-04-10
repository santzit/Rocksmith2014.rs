//! XML Files → SNG conversion tests.
//!
//! Mirrors `XmlFilesToSng.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{xml_to_sng, xml_vocals_to_sng, FontOption};
use rocksmith2014_xml::{glyph_definitions::GlyphDefinitions, read_file, vocal};
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Mirrors: testCase "Vocals (Default Font)"
///
/// Expect.equal sng.Vocals.Length xml.Count "Vocal count is same"
/// Expect.equal sng.SymbolDefinitions.Length 192 "Symbol definition count is correct"
#[test]
fn vocals_default_font() {
    let xml = vocal::load(test_dir().join("vocals.xml")).expect("load vocals.xml");

    let sng = xml_vocals_to_sng(FontOption::DefaultFont, &xml);

    assert_eq!(sng.vocals.len(), xml.len(), "Vocal count is same");
    assert_eq!(
        sng.symbol_definitions.len(),
        192,
        "Symbol definition count is correct"
    );
}

/// Mirrors: testCase "Vocals (Custom Font)"
///
/// Expect.equal sng.Vocals.Length xml.Count "Vocal count is same"
/// Expect.equal sng.SymbolDefinitions.Length customFont.Glyphs.Count "Symbol definition count is correct"
/// Expect.equal sng.SymbolsTextures.[0].Width customFont.TextureWidth "Texture width is correct"
/// Expect.equal sng.SymbolsTextures.[0].Height customFont.TextureHeight "Texture height is correct"
#[test]
fn vocals_custom_font() {
    let xml = vocal::load(test_dir().join("vocals.xml")).expect("load vocals.xml");
    let custom_font = GlyphDefinitions::load(test_dir().join("vocals.glyphs.xml"))
        .expect("load vocals.glyphs.xml");

    let sng = xml_vocals_to_sng(FontOption::CustomFont(&custom_font, "nothing"), &xml);

    assert_eq!(sng.vocals.len(), xml.len(), "Vocal count is same");
    assert_eq!(
        sng.symbol_definitions.len(),
        custom_font.glyphs.len(),
        "Symbol definition count is correct"
    );
    assert_eq!(
        sng.symbols_textures[0].width, custom_font.texture_width,
        "Texture width is correct"
    );
    assert_eq!(
        sng.symbols_textures[0].height, custom_font.texture_height,
        "Texture height is correct"
    );
}

/// Mirrors: testAsync "Japanese Vocals (Custom Font)"
///
/// Round-trips Japanese vocals through SNG encryption and back, verifying
/// lyric content and symbol definitions are preserved.
///
/// Expect.equal sng.Vocals.Length xml.Count "Vocal count is same"
/// Expect.equal sng.Vocals.[0].Lyric "夏-" "Vocal #1 is correct"
/// Expect.equal sng.Vocals.[9].Lyric "跡+" "Vocal #9 is correct"
/// Expect.equal sng.SymbolDefinitions.Length customFont.Glyphs.Count "Symbol definition count is correct"
/// Expect.equal sng.SymbolDefinitions.[1].Symbol "が" "Symbol #1 is correct"
#[test]
fn japanese_vocals_custom_font() {
    use rocksmith2014_sng::{Platform, Sng};

    let xml = vocal::load(test_dir().join("jvocals.xml")).expect("load jvocals.xml");
    let custom_font = GlyphDefinitions::load(test_dir().join("jvocals.glyphs.xml"))
        .expect("load jvocals.glyphs.xml");

    let sng_in = xml_vocals_to_sng(FontOption::CustomFont(&custom_font, "nothing"), &xml);

    // Round-trip through SNG encryption (PC platform)
    let encrypted = sng_in.to_encrypted(Platform::Pc).expect("to_encrypted");
    let sng = Sng::from_encrypted(&encrypted, Platform::Pc).expect("from_encrypted");

    assert_eq!(sng.vocals.len(), xml.len(), "Vocal count is same");

    // Lyric bytes → string helper
    let lyric_str = |bytes: &[u8]| {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    };

    assert_eq!(
        lyric_str(&sng.vocals[0].lyric),
        "夏-",
        "Vocal #1 is correct"
    );
    assert_eq!(
        lyric_str(&sng.vocals[9].lyric),
        "跡+",
        "Vocal #9 is correct"
    );
    assert_eq!(
        sng.symbol_definitions.len(),
        custom_font.glyphs.len(),
        "Symbol definition count is correct"
    );

    let sym1_str = |bytes: &[u8]| {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    };
    assert_eq!(
        sym1_str(&sng.symbol_definitions[1].symbol),
        "が",
        "Symbol #1 is correct"
    );
}

/// Mirrors: testCase "Instrumental"
///
/// Expect.equal sng.Beats.Length xml.Ebeats.Count "Beat count is same"
/// Expect.equal sng.Phrases.Length xml.Phrases.Count "Phrase count is same"
/// Expect.equal sng.Chords.Length xml.ChordTemplates.Count "Chord template count is same"
/// Expect.equal sng.Vocals.Length 0 "Vocals count is zero"
/// Expect.equal sng.SymbolsHeaders.Length 0 "Symbol headers count is zero"
/// Expect.equal sng.SymbolsTextures.Length 0 "Symbol textures count is zero"
/// Expect.equal sng.SymbolDefinitions.Length 0 "Symbol definitions count is zero"
/// Expect.equal sng.PhraseIterations.Length xml.PhraseIterations.Count "Phrase iteration count is same"
/// Expect.equal sng.NewLinkedDifficulties.Length xml.NewLinkedDiffs.Count "Linked difficulties count is same"
/// Expect.equal sng.Events.Length xml.Events.Count "Event count is same"
/// Expect.equal sng.Tones.Length xml.Tones.Changes.Count "Tone change count is same"
/// Expect.equal sng.DNAs.Length 2 "DNA count is correct"
/// Expect.equal sng.Sections.Length xml.Sections.Count "Section count is same"
/// Expect.equal sng.Levels.Length xml.Levels.Count "Level count is same"
#[test]
fn instrumental() {
    let xml = read_file(test_dir().join("instrumental.xml")).expect("load instrumental.xml");

    let sng = xml_to_sng(&xml);

    assert_eq!(sng.beats.len(), xml.ebeats.len(), "Beat count is same");
    assert_eq!(sng.phrases.len(), xml.phrases.len(), "Phrase count is same");
    assert_eq!(
        sng.chords.len(),
        xml.chord_templates.len(),
        "Chord template count is same"
    );
    assert_eq!(sng.vocals.len(), 0, "Vocals count is zero");
    assert_eq!(sng.symbols_headers.len(), 0, "Symbol headers count is zero");
    assert_eq!(
        sng.symbols_textures.len(),
        0,
        "Symbol textures count is zero"
    );
    assert_eq!(
        sng.symbol_definitions.len(),
        0,
        "Symbol definitions count is zero"
    );
    assert_eq!(
        sng.phrase_iterations.len(),
        xml.phrase_iterations.len(),
        "Phrase iteration count is same"
    );
    assert_eq!(
        sng.new_linked_difficulties.len(),
        xml.linked_diffs.len(),
        "Linked difficulties count is same"
    );
    assert_eq!(sng.events.len(), xml.events.len(), "Event count is same");
    assert_eq!(
        sng.tones.len(),
        xml.tones.len(),
        "Tone change count is same"
    );
    assert_eq!(sng.dnas.len(), 2, "DNA count is correct");
    assert_eq!(
        sng.sections.len(),
        xml.sections.len(),
        "Section count is same"
    );
    assert_eq!(sng.levels.len(), xml.levels.len(), "Level count is same");
}
