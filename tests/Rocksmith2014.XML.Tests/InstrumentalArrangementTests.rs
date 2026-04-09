//! Tests for loading and manipulating [`InstrumentalArrangement`].
//!
//! Mirrors `InstrumentalArrangementTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::*;
use std::path::PathBuf;

fn xml_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Mirrors `InstrumentalArrangementTests.CanRemoveDD`.
#[test]
fn can_remove_dd() {
    let mut arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");
    arr.remove_dd();
    assert_eq!(arr.levels.len(), 1);
}

/// Round-trip: load → serialise → reload and confirm the arrangement is stable.
#[test]
fn instrumental_xml_roundtrip() {
    let arr = read_file(xml_dir().join("instrumental.xml")).expect("read instrumental.xml");
    let xml_str = arr.to_xml().expect("to_xml");
    let arr2 = InstrumentalArrangement::from_xml(&xml_str).expect("from_xml round-trip");

    assert_eq!(arr.levels.len(), arr2.levels.len(), "level count preserved");
    assert_eq!(arr.ebeats.len(), arr2.ebeats.len(), "ebeat count preserved");
    assert_eq!(
        arr.chord_templates.len(),
        arr2.chord_templates.len(),
        "chord template count preserved"
    );
    assert_eq!(
        arr.meta.song_name, arr2.meta.song_name,
        "song name preserved"
    );
}

/// `ChordTemplate::default()` uses -1 sentinels for frets and fingers.
#[test]
fn chord_template_default_uses_sentinel_values() {
    let ct = ChordTemplate::default();
    assert_eq!(ct.frets, [-1i8; 6]);
    assert_eq!(ct.fingers, [-1i8; 6]);
}

/// Note mask round-trips through XML serialisation.
#[test]
fn note_mask_roundtrip() {
    let note = Note {
        time: 1000,
        fret: 5,
        string: 0,
        mask: NoteMask::HAMMER_ON,
        ..Default::default()
    };
    let mut arr = InstrumentalArrangement::default();
    arr.levels.push(Level {
        difficulty: 0,
        notes: vec![note],
        ..Default::default()
    });

    let xml = arr.to_xml().unwrap();
    let arr2 = InstrumentalArrangement::from_xml(&xml).unwrap();
    assert!(arr2.levels[0].notes[0].mask.contains(NoteMask::HAMMER_ON));
}

/// Full arrangement with notes, phrases, and ebeats round-trips through XML.
#[test]
fn roundtrip_with_notes() {
    let mut arr = InstrumentalArrangement::default();
    arr.meta.arrangement = "Lead".to_string();
    arr.meta.song_name = "Test Song".to_string();
    arr.meta.artist_name = "Test Artist".to_string();
    arr.meta.album_name = "Test Album".to_string();
    arr.meta.song_length = 120000;
    arr.meta.average_tempo = 120.0;

    arr.phrases.push(Phrase {
        name: "phrase1".to_string(),
        max_difficulty: 5,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 0,
        end_time: 5000,
        phrase_id: 0,
        hero_levels: None,
    });
    arr.ebeats.push(Ebeat {
        time: 0,
        measure: 1,
    });
    arr.ebeats.push(Ebeat {
        time: 500,
        measure: -1,
    });

    let note = Note {
        time: 1000,
        fret: 7,
        string: 3,
        mask: NoteMask::HAMMER_ON,
        ..Default::default()
    };
    arr.levels.push(Level {
        difficulty: 0,
        notes: vec![note],
        ..Default::default()
    });

    let xml = arr.to_xml().unwrap();
    let arr2 = InstrumentalArrangement::from_xml(&xml).unwrap();

    assert_eq!(arr2.meta.arrangement, "Lead");
    assert_eq!(arr2.phrases.len(), 1);
    assert_eq!(arr2.ebeats.len(), 2);
    assert_eq!(arr2.ebeats[0].time, 0);
    assert_eq!(arr2.ebeats[1].measure, -1);
    assert_eq!(arr2.levels[0].notes[0].fret, 7);
    assert!(arr2.levels[0].notes[0].mask.contains(NoteMask::HAMMER_ON));
}
