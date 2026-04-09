//! Vocals round-trip conversion tests.
//!
//! Mirrors `VocalsConversion.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{sng_vocals_to_xml, xml_vocals_to_sng, FontOption};
use rocksmith2014_sng::{Platform, Sng};
use rocksmith2014_xml::Vocal;

fn round_trip(vocals: &[Vocal]) -> Vec<Vocal> {
    let sng = xml_vocals_to_sng(FontOption::DefaultFont, vocals);
    let encrypted = sng.to_encrypted(Platform::Pc).expect("to_encrypted");
    let sng2 = Sng::from_encrypted(&encrypted, Platform::Pc).expect("from_encrypted");
    sng_vocals_to_xml(&sng2)
}

/// Mirrors: testAsync "Simple lyrics round trip conversion"
///
/// Expect.hasLength xml 2 "Count of vocals is same"
/// Expect.equal xml.[0].Lyric "Test+" "Lyric is same"
#[test]
fn simple_lyrics_round_trip_conversion() {
    let source = vec![
        Vocal::new(5000, 100, "Test+", 254),
        Vocal::new(5400, 100, "Lyrics+", 254),
    ];

    let xml = round_trip(&source);

    assert_eq!(xml.len(), 2, "Count of vocals is same");
    assert_eq!(xml[0].lyric, "Test+", "Lyric is same");
}

/// Mirrors: testAsync "Lyrics round trip conversion truncates long lyric"
///
/// The space allowed for a single lyric is 48 bytes in the SNG format.
/// With the necessary null terminator, the resulting length should be 47.
///
/// Expect.equal xml.[0].Lyric.Length 47 "Lyric was truncated to 47 characters"
/// Expect.equal xml.[0].Lyric "This line of lyrics is more than 48 characters " "Lyric string was truncated"
#[test]
fn lyrics_round_trip_conversion_truncates_long_lyric() {
    let source = vec![Vocal::new(
        5000,
        100,
        "This line of lyrics is more than 48 characters when encoded in UTF8.",
        254,
    )];

    let xml = round_trip(&source);

    assert_eq!(
        xml[0].lyric.len(),
        47,
        "Lyric was truncated to 47 characters"
    );
    assert_eq!(
        xml[0].lyric, "This line of lyrics is more than 48 characters ",
        "Lyric string was truncated"
    );
}
