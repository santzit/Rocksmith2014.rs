//! Integration tests for the rocksmith2014-conversion crate.
//!
//! Mirrors the .NET `Rocksmith2014.Conversion.Tests` project, testing
//! bidirectional conversion between SNG and XML arrangement formats.

use rocksmith2014_conversion::{sng_to_xml_full, to_midi_note, xml_to_sng};
use rocksmith2014_xml::{
    Anchor, Ebeat, InstrumentalArrangement, Level, MetaData, Note, Phrase, PhraseIteration,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_arr() -> InstrumentalArrangement {
    InstrumentalArrangement {
        phrases: vec![
            Phrase {
                name: "COUNT".into(),
                ..Default::default()
            },
            Phrase {
                name: "riff".into(),
                max_difficulty: 0,
                ..Default::default()
            },
            Phrase {
                name: "END".into(),
                ..Default::default()
            },
        ],
        phrase_iterations: vec![
            PhraseIteration {
                time: 0,
                end_time: 2000,
                phrase_id: 0,
                ..Default::default()
            },
            PhraseIteration {
                time: 2000,
                end_time: 8000,
                phrase_id: 1,
                ..Default::default()
            },
            PhraseIteration {
                time: 8000,
                end_time: 10000,
                phrase_id: 2,
                ..Default::default()
            },
        ],
        ebeats: vec![
            Ebeat {
                time: 0,
                measure: 0,
            },
            Ebeat {
                time: 500,
                measure: -1,
            },
            Ebeat {
                time: 1000,
                measure: 1,
            },
        ],
        sections: vec![],
        levels: vec![Level {
            difficulty: 0,
            notes: vec![Note {
                time: 2000,
                string: 2,
                fret: 5,
                sustain: 500,
                ..Default::default()
            }],
            anchors: vec![Anchor {
                time: 2000,
                end_time: 8000,
                fret: 5,
                width: 4,
            }],
            ..Default::default()
        }],
        meta: MetaData {
            song_length: 10_000,
            ..Default::default()
        },
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// XML → SNG conversion
// ---------------------------------------------------------------------------

/// Phrases are correctly converted to SNG format.
#[test]
fn xml_to_sng_phrases_count_matches() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    assert_eq!(sng.phrases.len(), arr.phrases.len());
}

/// Beats (ebeats) are converted to SNG beats with correct measure numbers.
#[test]
fn xml_to_sng_beats_count_matches() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    assert_eq!(sng.beats.len(), arr.ebeats.len());
}

/// Phrase iterations are converted and count matches.
#[test]
fn xml_to_sng_phrase_iterations_count_matches() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    assert_eq!(sng.phrase_iterations.len(), arr.phrase_iterations.len());
}

/// Notes in the arrangement level are converted to SNG notes.
#[test]
fn xml_to_sng_notes_are_converted() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    let sng_level = sng
        .levels
        .first()
        .expect("SNG should have at least one level");
    assert_eq!(
        sng_level.notes.len(),
        arr.levels[0].notes.len(),
        "Note count should match"
    );
}

/// Note time is converted from milliseconds (XML) to seconds (SNG).
#[test]
fn xml_to_sng_note_time_converted_to_seconds() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    let note = &sng.levels[0].notes[0];
    // XML note time is 2000 ms → SNG note time should be ~2.0 s
    assert!(
        (note.time - 2.0).abs() < 0.001,
        "Note time should be ~2.0 s, got {}",
        note.time
    );
}

/// Anchors are converted from XML to SNG.
#[test]
fn xml_to_sng_anchors_are_converted() {
    let arr = minimal_arr();
    let sng = xml_to_sng(&arr);
    let sng_level = sng
        .levels
        .first()
        .expect("SNG should have at least one level");
    assert_eq!(sng_level.anchors.len(), arr.levels[0].anchors.len());
}

// ---------------------------------------------------------------------------
// SNG → XML conversion
// ---------------------------------------------------------------------------

/// Round-trip: XML → SNG → XML. The phrase count must be preserved.
#[test]
fn round_trip_preserves_phrase_count() {
    let orig = minimal_arr();
    let sng = xml_to_sng(&orig);
    let back = sng_to_xml_full(&sng);
    assert_eq!(
        back.phrases.len(),
        orig.phrases.len(),
        "Phrase count should survive round-trip"
    );
}

/// Round-trip: beat count is preserved.
#[test]
fn round_trip_preserves_beat_count() {
    let orig = minimal_arr();
    let sng = xml_to_sng(&orig);
    let back = sng_to_xml_full(&sng);
    assert_eq!(
        back.ebeats.len(),
        orig.ebeats.len(),
        "Beat count should survive round-trip"
    );
}

/// Round-trip: level count is preserved.
#[test]
fn round_trip_preserves_level_count() {
    let orig = minimal_arr();
    let sng = xml_to_sng(&orig);
    let back = sng_to_xml_full(&sng);
    assert_eq!(
        back.levels.len(),
        orig.levels.len(),
        "Level count should survive round-trip"
    );
}

/// Round-trip: note count in the first level is preserved.
#[test]
fn round_trip_preserves_note_count() {
    let orig = minimal_arr();
    let sng = xml_to_sng(&orig);
    let back = sng_to_xml_full(&sng);
    assert_eq!(
        back.levels[0].notes.len(),
        orig.levels[0].notes.len(),
        "Note count should survive round-trip"
    );
}

/// Round-trip: note time is preserved (within rounding tolerance).
#[test]
fn round_trip_note_time_within_tolerance() {
    let orig = minimal_arr();
    let sng = xml_to_sng(&orig);
    let back = sng_to_xml_full(&sng);
    let orig_ms = orig.levels[0].notes[0].time;
    let back_ms = back.levels[0].notes[0].time;
    assert!(
        (back_ms - orig_ms).abs() <= 1,
        "Note time should round-trip within 1 ms, orig={orig_ms} back={back_ms}"
    );
}

// ---------------------------------------------------------------------------
// MIDI note calculation
// ---------------------------------------------------------------------------

/// Standard tuning open strings match expected MIDI notes.
///
/// String 0 (low E2) open = MIDI 40; tuning offset 0; no capo.
#[test]
fn midi_note_open_string_standard_tuning() {
    let tuning = [0i16; 6];
    // String 0 (E2) open → MIDI 40
    assert_eq!(to_midi_note(0, 0, &tuning, 0, false), 40);
    // String 5 (E4) open → MIDI 64
    assert_eq!(to_midi_note(5, 0, &tuning, 0, false), 64);
}

/// Fret offset is correctly added to the open-string MIDI note.
#[test]
fn midi_note_fret_offset() {
    let tuning = [0i16; 6];
    // String 0 (E2), fret 5 → MIDI 40 + 5 = 45
    assert_eq!(to_midi_note(0, 5, &tuning, 0, false), 45);
}

/// Capo affects open-string (fret 0) MIDI note.
#[test]
fn midi_note_capo_affects_open_string() {
    let tuning = [0i16; 6];
    // Capo on fret 3, playing "open" (fret 0) → treated as fret 3
    assert_eq!(to_midi_note(0, 0, &tuning, 3, false), 43);
}

/// Bass arrangements are transposed down one octave (−12 semitones).
#[test]
fn midi_note_bass_is_octave_lower() {
    let tuning = [0i16; 6];
    let guitar = to_midi_note(0, 0, &tuning, 0, false);
    let bass = to_midi_note(0, 0, &tuning, 0, true);
    assert_eq!(
        guitar - bass,
        12,
        "Bass should be 1 octave (12 semitones) lower"
    );
}

/// Non-zero tuning offsets are applied correctly.
#[test]
fn midi_note_tuning_offset_applied() {
    let mut tuning = [0i16; 6];
    tuning[0] = -2; // Drop-D style: string 0 down 2 semitones
                    // String 0 open with -2 offset → MIDI 40 - 2 = 38
    assert_eq!(to_midi_note(0, 0, &tuning, 0, false), 38);
}
