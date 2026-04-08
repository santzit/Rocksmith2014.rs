//! Tests for reading [`MetaData`] from an XML file.
//!
//! Mirrors `MetaDataTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::read_file;
use std::path::PathBuf;

fn xml_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Mirrors `MetaDataTests.CanBeReadFromXMLFile`:
/// - Title == "Test Instrumental"
/// - AverageTempo ≈ 160.541
/// - ArtistNameSort == "Test"
/// - LastConversionDateTime == "5-17-20 15:21"
#[test]
fn can_be_read_from_xml_file() {
    let arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");

    assert_eq!(arr.meta.song_name, "Test Instrumental");
    assert!(
        (arr.meta.average_tempo - 160.541).abs() < 0.001,
        "average_tempo should be ~160.541, got {}",
        arr.meta.average_tempo
    );
    assert_eq!(arr.meta.artist_name_sort, "Test");
    assert_eq!(arr.meta.last_conversion_date_time, "5-17-20 15:21");
}
