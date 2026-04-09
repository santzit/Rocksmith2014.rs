//! SNG Files → XML conversion tests.
//!
//! Mirrors `SngFilesToXml.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{extract_glyph_data, sng_to_xml_full, sng_vocals_to_xml};
use rocksmith2014_sng::{Platform, Sng};
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Helper that mirrors `Utils.TimeCodeFromFloatString`: converts seconds (f32) to
/// milliseconds (i32) without floating-point arithmetic issues.
fn convert_time(time: f32) -> i32 {
    (time.to_string().parse::<f64>().unwrap_or(time as f64) * 1000.0).round() as i32
}

/// Mirrors: testAsync "Vocals"
///
/// Expect.equal xml.Count sng.Vocals.Length "Vocals count is same"
/// for each vocal: Lyric, Note, Time, Length are all same
#[test]
fn vocals() {
    let data = std::fs::read(test_dir().join("vocals.sng")).expect("read vocals.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("parse vocals.sng");

    let xml = sng_vocals_to_xml(&sng);

    assert_eq!(xml.len(), sng.vocals.len(), "Vocals count is same");

    let lyric_str = |bytes: &[u8; 48]| {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(48);
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    };

    for (i, xml_vocal) in xml.iter().enumerate() {
        assert_eq!(
            xml_vocal.lyric,
            lyric_str(&sng.vocals[i].lyric),
            "Lyric #{i} is same"
        );
        assert_eq!(
            xml_vocal.note, sng.vocals[i].note as u8,
            "Note #{i} is same"
        );
        assert_eq!(
            xml_vocal.time,
            convert_time(sng.vocals[i].time),
            "Time #{i} is same"
        );
        assert_eq!(
            xml_vocal.length,
            convert_time(sng.vocals[i].length),
            "Length #{i} is same"
        );
    }
}

/// Mirrors: testAsync "Extract Glyphs"
///
/// Expect.equal xml.Glyphs.Count sng.SymbolDefinitions.Length "Same glyph count"
/// Expect.equal xml.TextureWidth sng.SymbolsTextures.[0].Width "Same texture width"
/// Expect.equal xml.TextureHeight sng.SymbolsTextures.[0].Height "Same texture height"
#[test]
fn extract_glyphs() {
    let data = std::fs::read(test_dir().join("vocals.sng")).expect("read vocals.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("parse vocals.sng");

    let xml = extract_glyph_data(&sng);

    assert_eq!(
        xml.glyphs.len(),
        sng.symbol_definitions.len(),
        "Same glyph count"
    );
    assert_eq!(
        xml.texture_width, sng.symbols_textures[0].width,
        "Same texture width"
    );
    assert_eq!(
        xml.texture_height, sng.symbols_textures[0].height,
        "Same texture height"
    );
}

/// Mirrors: testAsync "Instrumental"
///
/// Expect.equal xml.MetaData.Part sng.MetaData.Part "Same part"
/// Expect.equal xml.MetaData.Capo 0y "Capo fret -1 in SNG is 0 in XML"
/// Expect.equal xml.MetaData.LastConversionDateTime sng.MetaData.LastConversionDateTime "Same last conversion date"
/// Expect.sequenceEqual xml.MetaData.Tuning.Strings sng.MetaData.Tuning "Same tuning"
/// Expect.equal xml.MetaData.SongLength (convertTime sng.MetaData.SongLength) "Same song length"
/// Expect.equal xml.Phrases.Count sng.Phrases.Length "Same phrase count"
/// Expect.equal xml.PhraseIterations.Count sng.PhraseIterations.Length "Same phrase iteration count"
/// Expect.equal xml.NewLinkedDiffs.Count sng.NewLinkedDifficulties.Length "Same new linked difficulties count"
/// Expect.equal xml.ChordTemplates.Count sng.Chords.Length "Same chord template count"
/// Expect.equal xml.Ebeats.Count sng.Beats.Length "Same beat count"
/// Expect.equal xml.Tones.Changes.Count sng.Tones.Length "Same tone count"
/// Expect.equal xml.Sections.Count sng.Sections.Length "Same section count"
/// Expect.equal xml.Events.Count sng.Events.Length "Same event count"
/// Expect.equal xml.Levels.Count sng.Levels.Length "Same level count"
#[test]
fn instrumental() {
    let data = std::fs::read(test_dir().join("instrumental.sng")).expect("read instrumental.sng");
    let sng = Sng::from_encrypted(&data, Platform::Pc).expect("parse instrumental.sng");

    let xml = sng_to_xml_full(&sng);

    let lcd_sng = {
        let end = sng
            .metadata
            .last_conversion_date_time
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(32);
        String::from_utf8_lossy(&sng.metadata.last_conversion_date_time[..end]).into_owned()
    };

    assert_eq!(xml.meta.part, sng.metadata.part as i32, "Same part");
    assert_eq!(xml.meta.capo, 0, "Capo fret -1 in SNG is 0 in XML");
    assert_eq!(
        xml.meta.last_conversion_date_time, lcd_sng,
        "Same last conversion date"
    );
    assert_eq!(
        xml.meta.tuning.strings.as_slice(),
        sng.metadata.tuning.as_slice(),
        "Same tuning"
    );
    assert_eq!(
        xml.meta.song_length,
        convert_time(sng.metadata.song_length),
        "Same song length"
    );
    assert_eq!(xml.phrases.len(), sng.phrases.len(), "Same phrase count");
    assert_eq!(
        xml.phrase_iterations.len(),
        sng.phrase_iterations.len(),
        "Same phrase iteration count"
    );
    assert_eq!(
        xml.linked_diffs.len(),
        sng.new_linked_difficulties.len(),
        "Same new linked difficulties count"
    );
    assert_eq!(
        xml.chord_templates.len(),
        sng.chords.len(),
        "Same chord template count"
    );
    assert_eq!(xml.ebeats.len(), sng.beats.len(), "Same beat count");
    assert_eq!(xml.tones.len(), sng.tones.len(), "Same tone count");
    assert_eq!(xml.sections.len(), sng.sections.len(), "Same section count");
    assert_eq!(xml.events.len(), sng.events.len(), "Same event count");
    assert_eq!(xml.levels.len(), sng.levels.len(), "Same level count");

    if !sng.phrase_extra_info.is_empty() {
        assert_eq!(
            xml.phrase_properties.len(),
            sng.phrase_extra_info.len(),
            "Same phrase property count"
        );
    }
}
