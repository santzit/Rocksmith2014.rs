//! Tests for Vocal read/write operations.
//!
//! Mirrors `VocalTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::vocal::{self, Vocal};

#[test]
fn copy_constructor_copies_all_values() {
    let v1 = Vocal::new(12345, 500, "test", 66);
    let v2 = Vocal::copy(&v1);

    assert_eq!(v2.time, 12345);
    assert_eq!(v2.length, 500);
    assert_eq!(v2.lyric, "test");
    assert_eq!(v2.note, 66);
}

#[test]
fn list_of_vocals_can_be_saved_to_xml_file() {
    let vocals = vec![
        Vocal::default(),
        Vocal::new(12340, 500, "Test", 66),
        Vocal::new(25678, 500, "Test 2", 66),
    ];

    let path = std::env::temp_dir().join("vocals_save_test.xml");
    vocal::save(&path, &vocals).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains("<vocals count=\"3\">"));
    assert!(content.contains("time=\"12.340\" note=\"66\" length=\"0.500\" lyric=\"Test\""));
}

#[test]
fn list_of_vocals_can_be_read_from_xml_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("Vocals.xml");

    let vocals = vocal::load(&path).unwrap();

    assert_eq!(vocals.len(), 8);

    // <vocal time="28.780" note="254" length="0.600" lyric="sum+"/>
    assert_eq!(vocals[5].time, 28_780);
    assert_eq!(vocals[5].note, 254);
    assert_eq!(vocals[5].length, 600);
    assert_eq!(vocals[5].lyric, "sum+");
}

