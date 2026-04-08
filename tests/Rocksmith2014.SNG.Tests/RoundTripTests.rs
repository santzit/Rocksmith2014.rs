//! Mirrors RoundTripTests.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::{Anchor, Beat, BeatMask, Level, Note, NoteMask, Platform, Sng};

#[test]
fn test_beat_roundtrip() {
    let mut sng = Sng::default();
    sng.beats.push(Beat {
        time: 1.0,
        measure: 1,
        beat: 0,
        phrase_iteration: 0,
        mask: BeatMask::FIRST_BEAT_OF_MEASURE,
    });
    let bytes = sng.write().unwrap();
    let sng2 = Sng::read(&bytes).unwrap();
    assert_eq!(sng2.beats.len(), 1);
    assert_eq!(sng2.beats[0].time, 1.0);
    assert_eq!(sng2.beats[0].mask, BeatMask::FIRST_BEAT_OF_MEASURE);
}

#[test]
fn test_note_roundtrip() {
    let note = Note {
        mask: NoteMask::SINGLE,
        fret: 5,
        ..Note::default()
    };
    let mut sng = Sng::default();
    let level = Level {
        notes: vec![note],
        ..Level::default()
    };
    sng.levels.push(level);
    let bytes = sng.write().unwrap();
    let sng2 = Sng::read(&bytes).unwrap();
    assert_eq!(sng2.levels[0].notes[0].fret, 5);
    assert_eq!(sng2.levels[0].notes[0].mask, NoteMask::SINGLE);
}

#[test]
fn test_level_roundtrip() {
    let level = Level {
        difficulty: 0,
        anchors: vec![Anchor {
            start_time: 0.0,
            end_time: 10.0,
            first_note_time: 0.5,
            last_note_time: 9.5,
            fret_id: 1,
            width: 4,
            phrase_iteration_id: 0,
        }],
        average_notes_per_iteration: vec![1.0],
        notes_in_phrase_iterations_excl_ignored: vec![1],
        notes_in_phrase_iterations_all: vec![1],
        ..Level::default()
    };
    let mut sng = Sng::default();
    sng.levels.push(level);
    let bytes = sng.write().unwrap();
    let sng2 = Sng::read(&bytes).unwrap();
    assert_eq!(sng2.levels[0].anchors[0].fret_id, 1);
}

#[test]
fn test_empty_sng_roundtrip() {
    let sng = Sng::default();
    let bytes = sng.write().unwrap();
    let sng2 = Sng::read(&bytes).unwrap();
    assert_eq!(sng2.beats.len(), 0);
    assert_eq!(sng2.levels.len(), 0);
}

#[test]
fn test_encrypted_roundtrip() {
    let sng = Sng::default();
    let enc = sng.to_encrypted(Platform::Pc).unwrap();
    let sng2 = Sng::from_encrypted(&enc, Platform::Pc).unwrap();
    assert_eq!(sng2.beats.len(), 0);
    assert_eq!(sng2.metadata.max_score, 0.0);
}
