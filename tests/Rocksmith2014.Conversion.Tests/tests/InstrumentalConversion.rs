//! Instrumental XML → SNG conversion tests.
//!
//! Mirrors `InstrumentalConversion.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::xml_to_sng;
use rocksmith2014_sng::NoteMask;
use rocksmith2014_xml::read_file;
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Mirrors: testCase "Notes Only"
#[test]
fn notes_only() {
    let xml = read_file(test_dir().join("instrumental_1level_notesonly.xml")).expect("load xml");

    let sng = xml_to_sng(&xml);
    let level = &sng.levels[0];

    // Test note counts
    assert_eq!(
        sng.metadata.max_notes_and_chords, 17.0,
        "Total number of notes is 17"
    );
    assert_eq!(
        sng.metadata.max_notes_and_chords_real, 16.0,
        "Total number of notes - ignored notes is 16"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_all[1], 10,
        "Number of notes in phrase iteration #1 is 10"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_all[2], 7,
        "Number of notes in phrase iteration #2 is 7"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_excl_ignored[2], 6,
        "Number of notes (excluding ignored) in phrase iteration #2 is 6"
    );

    // Test beat phrase iterations
    assert_eq!(
        sng.beats[4].phrase_iteration, 0,
        "Beat #4 for is in phrase iteration 0"
    );
    assert_eq!(
        sng.beats[5].phrase_iteration, 1,
        "Beat #5 for is in phrase iteration 1"
    );
    assert_eq!(
        sng.beats[12].phrase_iteration, 1,
        "Beat #12 for is in phrase iteration 1"
    );
    assert_eq!(
        sng.beats[13].phrase_iteration, 2,
        "Beat #13 for is in phrase iteration 2"
    );

    // Test various properties of the notes
    assert_eq!(
        level.notes[0].anchor_fret, 2,
        "Note #0 is anchored on fret 2"
    );
    assert!(
        level.notes[2].mask.contains(NoteMask::OPEN),
        "Note #2 has open bit set"
    );
    assert_eq!(
        level.notes[6].finger_print_id[1], 0,
        "Note #6 is inside arpeggio (Chord ID 0)"
    );
    assert_eq!(level.notes[9].sustain, 0.750, "Note #9 has 0.750s sustain");
    assert_eq!(level.notes[10].max_bend, 1.0, "Note #10 max bend is 1.0");
    assert_eq!(
        level.notes[10].bend_data.len(),
        1,
        "Note #10 has one bend value"
    );
    assert_eq!(
        level.notes[11].slide_unpitch_to, 14,
        "Note #11 has unpitched slide to fret 14"
    );
    assert!(
        level.notes[15].mask.contains(NoteMask::PARENT),
        "Note #15 has parent bit set"
    );
    assert_eq!(
        level.notes[16].vibrato, 80,
        "Note #16 has vibrato set to 80"
    );
}

/// Mirrors: testCase "Chords Only"
#[test]
fn chords_only() {
    let xml = read_file(test_dir().join("instrumental_1level_chordsonly.xml")).expect("load xml");

    let sng = xml_to_sng(&xml);
    let level = &sng.levels[0];

    // Test note counts
    assert_eq!(
        sng.metadata.max_notes_and_chords, 8.0,
        "Total number of notes is 8"
    );
    assert_eq!(
        sng.metadata.max_notes_and_chords_real, 7.0,
        "Total number of notes - ignored notes is 7"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_all[1], 7,
        "Number of notes in phrase iteration #1 is 7"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_all[2], 1,
        "Number of notes in phrase iteration #2 is 1"
    );
    assert_eq!(
        level.notes_in_phrase_iterations_excl_ignored[2], 0,
        "Number of notes (excluding ignored) in phrase iteration #2 is 0"
    );

    // Test chord notes
    assert_eq!(
        sng.chord_notes.len(),
        2,
        "Number of chord notes generated is 2"
    );
    assert!(
        sng.chord_notes[0].mask[3] & NoteMask::OPEN.bits() != 0,
        "Chord notes #0 has open bit set on string 3"
    );
    assert!(
        sng.chord_notes[0].mask[4] & NoteMask::OPEN.bits() != 0,
        "Chord notes #0 has open bit set on string 4"
    );
    assert!(
        sng.chord_notes[1].mask[2] & NoteMask::SUSTAIN.bits() != 0,
        "Chord notes #1 has sustain bit set on string 2"
    );

    // Test various properties of the chords
    assert_eq!(
        level.notes[0].finger_print_id[0], 0,
        "Chord #0 is inside hand shape (Chord ID 0)"
    );
    assert!(
        level.notes[0].mask.contains(NoteMask::CHORD_PANEL),
        "Chord #0 has chord panel bit set"
    );
    assert_eq!(
        level.notes[1].finger_print_id[0], 0,
        "Chord #1 is inside hand shape (Chord ID 0)"
    );
    assert!(
        !level.notes[2].mask.contains(NoteMask::CHORD_PANEL),
        "Chord #2 does not have chord panel bit set"
    );
    assert_eq!(
        level.notes[4].finger_print_id[0], 1,
        "Chord #4 is inside hand shape (Chord ID 1)"
    );
    assert!(
        level.notes[4].mask.contains(NoteMask::DOUBLE_STOP),
        "Chord #4 has double stop bit set"
    );
    assert_eq!(
        level.notes[6].finger_print_id[0], 2,
        "Chord #6 is inside hand shape (Chord ID 2)"
    );
    assert_eq!(level.notes[6].sustain, 0.750, "Chord #6 has 0.75s sustain");
    assert!(
        level.notes[7].mask.contains(NoteMask::IGNORE),
        "Chord #7 has ignore bit set"
    );
}

/// Mirrors: testCase "Chord notes whose hash values may clash"
///
/// Expect.hasLength sng.ChordNotes 2 "Two chord notes were created"
#[test]
fn chord_notes_hash_clash() {
    let xml = read_file(test_dir().join("chordnotes.xml")).expect("load chordnotes.xml");

    let sng = xml_to_sng(&xml);

    assert_eq!(sng.chord_notes.len(), 2, "Two chord notes were created");
}
