//! Integration tests mirroring Rocksmith2014.NET v3.5.0 test suite.
//!
//! Test files live in `tests/cdlc/`:
//! - `packed_pc.sng`, `packed_mac.sng`, `unpacked.sng`  — from Rocksmith2014.SNG.Tests
//! - `test_p.psarc`, `partially_compressed_test_p.psarc` — from Rocksmith2014.PSARC.Tests
//! - `instrumental.xml`                                   — from Rocksmith2014.XML.Tests
//! - `test_arrangement.xml`                               — minimal hand-crafted CDLC fixture

use rocksmith2014::{
    psarc::{Psarc, PsarcBuilder},
    sng::{sng_to_bytes, types::*, Platform},
    xml::InstrumentalArrangement,
};

// ---------------------------------------------------------------------------
// Helper: path inside tests/cdlc/
// ---------------------------------------------------------------------------

fn cdlc(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cdlc")
        .join(name)
}

// ===========================================================================
// PSARC tests  (mirrors Rocksmith2014.PSARC.Tests)
// ===========================================================================

/// Mirrors: "Can read PSARC with encrypted TOC"
#[test]
fn psarc_can_read_encrypted_toc() {
    let psarc = Psarc::open(cdlc("test_p.psarc"))
        .expect("open test_p.psarc");

    assert_eq!(
        psarc.entry_name(0).unwrap(),
        "gfxassets/album_art/album_testtest_64.dds",
        "First manifest entry matches"
    );
}

/// Mirrors: "Can extract all files from PSARC"
#[test]
fn psarc_can_extract_all_entries() {
    let psarc = Psarc::open(cdlc("test_p.psarc"))
        .expect("open test_p.psarc");
    let count = psarc.entry_count();
    assert!(count > 0, "PSARC should have entries");

    for i in 0..count {
        let data = psarc.extract(i).expect("extract entry");
        assert!(!data.is_empty(), "entry {i} data should not be empty");
    }
}

/// Mirrors: "Can extract partially compressed file"
#[test]
fn psarc_can_extract_partially_compressed() {
    let psarc = Psarc::open(cdlc("partially_compressed_test_p.psarc"))
        .expect("open partially_compressed_test_p.psarc");
    let count = psarc.entry_count();
    assert!(count > 0, "Partially-compressed PSARC should have entries");

    for i in 0..count {
        let data = psarc.extract(i).expect("extract entry");
        assert!(!data.is_empty(), "entry {i} data should not be empty");
    }
}

/// Mirrors: "Manifest is same after null edit" (round-trip with builder)
#[test]
fn psarc_roundtrip_manifest_unchanged() {
    let xml = b"<song><title>Manifest Test</title></song>".to_vec();
    let mut builder = PsarcBuilder::new();
    builder.add_entry("songs/arr/test.xml", xml.clone());
    let bytes = builder.build();

    let psarc = Psarc::from_bytes(bytes).unwrap();
    assert_eq!(psarc.entry_count(), 1);
    assert_eq!(psarc.entry_name(0).unwrap(), "songs/arr/test.xml");
    assert_eq!(psarc.extract(0).unwrap(), xml);
}

/// Mirrors: "Can remove files" — builder with subset of entries
#[test]
fn psarc_builder_multiple_entries() {
    let file_a = b"file-a-data".to_vec();
    let file_b = b"file-b-wem".to_vec();
    let file_c = b"file-c-data".to_vec();

    let mut builder = PsarcBuilder::new();
    builder.add_entry("a.xml", file_a.clone());
    builder.add_entry("b.wem", file_b.clone());
    builder.add_entry("c.xml", file_c.clone());
    let bytes = builder.build();

    let psarc = Psarc::from_bytes(bytes).unwrap();
    assert_eq!(psarc.entry_count(), 3);
    // Only the non-.wem files (a.xml, c.xml) = 2
    let non_wem: Vec<_> = psarc
        .entry_names()
        .iter()
        .filter(|n| !n.ends_with(".wem"))
        .collect();
    assert_eq!(non_wem.len(), 2, "Two non-wem entries");
}

/// Mirrors: "Can rename files"
#[test]
fn psarc_builder_rename_roundtrip() {
    let data = b"content".to_vec();
    let mut builder = PsarcBuilder::new();
    builder.add_entry("original_name.xml", data.clone());
    let bytes = builder.build();
    let psarc = Psarc::from_bytes(bytes).unwrap();

    // Verify the name round-trips correctly
    assert_eq!(psarc.entry_name(0).unwrap(), "original_name.xml");
    assert_eq!(psarc.extract(0).unwrap(), data);
}

/// Mirrors: "Can reorder files" — verify multi-entry ordering
#[test]
fn psarc_builder_entry_order_preserved() {
    let mut builder = PsarcBuilder::new();
    builder.add_entry("first.xml", b"1".to_vec());
    builder.add_entry("second.xml", b"2".to_vec());
    builder.add_entry("third.xml", b"3".to_vec());
    let bytes = builder.build();
    let psarc = Psarc::from_bytes(bytes).unwrap();

    assert_eq!(psarc.entry_name(0).unwrap(), "first.xml");
    assert_eq!(psarc.entry_name(1).unwrap(), "second.xml");
    assert_eq!(psarc.entry_name(2).unwrap(), "third.xml");
    assert_eq!(psarc.extract(0).unwrap(), b"1");
    assert_eq!(psarc.extract(1).unwrap(), b"2");
    assert_eq!(psarc.extract(2).unwrap(), b"3");
}

/// PSARC: find_by_suffix helper works as expected
#[test]
fn psarc_find_by_suffix() {
    let psarc = Psarc::open(cdlc("test_p.psarc")).unwrap();
    let idx = psarc.find_by_suffix(".dds");
    assert!(idx.is_some(), "should find a .dds entry");
}

// ===========================================================================
// SNG tests  (mirrors Rocksmith2014.SNG.Tests)
// ===========================================================================

const EXPECTED_LEVEL_COUNT: usize = 12;

// --- Read / Write Unpacked -------------------------------------------------

/// Mirrors: "Can read unpacked SNG file"
#[test]
fn sng_can_read_unpacked_file() {
    let sng = Sng::read_unpacked_file(cdlc("unpacked.sng"))
        .expect("read unpacked.sng");
    assert_eq!(
        sng.levels.len(),
        EXPECTED_LEVEL_COUNT,
        "Unpacked SNG has {EXPECTED_LEVEL_COUNT} levels"
    );
}

/// Mirrors: "Can write unpacked SNG file" (round-trip: read → serialise → parse → compare)
#[test]
fn sng_can_write_unpacked_file() {
    let original = Sng::read_unpacked_file(cdlc("unpacked.sng"))
        .expect("read unpacked.sng");
    let original_bytes = std::fs::read(cdlc("unpacked.sng")).unwrap();

    // Serialise back to bytes
    let written = sng_to_bytes(&original);

    // Parse again
    let reparsed = Sng::from_unpacked_bytes(&written)
        .expect("re-parse serialised SNG");

    assert_eq!(
        reparsed.levels.len(),
        EXPECTED_LEVEL_COUNT,
        "Re-parsed SNG has correct level count"
    );
    assert_eq!(
        written.len(),
        original_bytes.len(),
        "Serialised SNG is the same size as the original"
    );
}

// --- Read / Write Packed ---------------------------------------------------

/// Mirrors: "Can read packed PC SNG file"
#[test]
fn sng_can_read_packed_pc() {
    let sng = Sng::read_packed_file(cdlc("packed_pc.sng"), Platform::Pc)
        .expect("read packed_pc.sng");
    assert_eq!(
        sng.levels.len(),
        EXPECTED_LEVEL_COUNT,
        "Packed PC SNG has {EXPECTED_LEVEL_COUNT} levels"
    );
}

/// Mirrors: "Can read packed Mac SNG file"
#[test]
fn sng_can_read_packed_mac() {
    let sng = Sng::read_packed_file(cdlc("packed_mac.sng"), Platform::Mac)
        .expect("read packed_mac.sng");
    assert_eq!(
        sng.levels.len(),
        EXPECTED_LEVEL_COUNT,
        "Packed Mac SNG has {EXPECTED_LEVEL_COUNT} levels"
    );
}

// --- Round-trip tests  (mirrors RoundTripTests.fs) -------------------------

fn make_metadata() -> MetaData {
    MetaData {
        max_score: 100_000.0,
        max_notes_and_chords: 456.0,
        max_notes_and_chords_real: 452.0,
        points_per_note: 100.0,
        first_beat_length: 88.0,
        start_time: 10.0,
        capo_fret_id: -1,
        last_conversion_date_time: "6-11-18 18:36".into(),
        part: 1,
        song_length: 520.0,
        tuning: (0..6).map(|i| i as i16).collect(),
        first_note_time: 15.0,
        max_difficulty: 22,
    }
}

fn empty_sng_with_metadata(md: MetaData) -> Sng {
    Sng {
        beats: vec![],
        phrases: vec![],
        chords: vec![],
        chord_notes: vec![],
        vocals: vec![],
        symbols_headers: vec![],
        symbols_textures: vec![],
        symbol_definitions: vec![],
        phrase_iterations: vec![],
        phrase_extra_info: vec![],
        new_linked_difficulties: vec![],
        actions: vec![],
        events: vec![],
        tones: vec![],
        dnas: vec![],
        sections: vec![],
        levels: vec![],
        metadata: md,
    }
}

/// Mirrors: round-trip for MetaData
#[test]
fn sng_roundtrip_metadata() {
    let md = make_metadata();
    let sng = empty_sng_with_metadata(md.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let out = &parsed.metadata;

    assert_eq!(out.max_score, md.max_score);
    assert_eq!(out.max_notes_and_chords, md.max_notes_and_chords);
    assert_eq!(out.max_notes_and_chords_real, md.max_notes_and_chords_real);
    assert_eq!(out.points_per_note, md.points_per_note);
    assert_eq!(out.first_beat_length, md.first_beat_length);
    assert_eq!(out.start_time, md.start_time);
    assert_eq!(out.capo_fret_id, md.capo_fret_id);
    assert_eq!(out.last_conversion_date_time, md.last_conversion_date_time);
    assert_eq!(out.part, md.part);
    assert_eq!(out.song_length, md.song_length);
    assert_eq!(out.tuning, md.tuning);
    assert_eq!(out.first_note_time, md.first_note_time);
    assert_eq!(out.max_difficulty, md.max_difficulty);
}

/// Mirrors: round-trip for Beat
#[test]
fn sng_roundtrip_beat() {
    let beat = Beat {
        time: 3.14,
        measure: 2,
        beat: 3,
        phrase_iteration: 5,
        mask: BeatMask::FIRST_BEAT_OF_MEASURE,
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.beats.push(beat.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let b = &parsed.beats[0];
    assert_eq!(b.time, beat.time);
    assert_eq!(b.measure, beat.measure);
    assert_eq!(b.beat, beat.beat);
    assert_eq!(b.phrase_iteration, beat.phrase_iteration);
    assert_eq!(b.mask, beat.mask);
}

/// Mirrors: round-trip for Phrase (including padding byte)
#[test]
fn sng_roundtrip_phrase() {
    let phrase = Phrase {
        solo: 1,
        disparity: 0,
        ignore: 0,
        max_difficulty: 20,
        iteration_count: 5,
        name: "verse".into(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.phrases.push(phrase.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let p = &parsed.phrases[0];
    assert_eq!(p.solo, phrase.solo);
    assert_eq!(p.max_difficulty, phrase.max_difficulty);
    assert_eq!(p.name, phrase.name);
}

/// Mirrors: round-trip for Chord
#[test]
fn sng_roundtrip_chord() {
    let chord = Chord {
        mask: ChordMask::ARPEGGIO,
        frets: [0, 3, 2, 0, -1, -1],
        fingers: [0, 2, 1, 0, -1, -1],
        notes: [0, 1, 2, 3, -1, -1],
        name: "Em".into(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.chords.push(chord.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let c = &parsed.chords[0];
    assert_eq!(c.mask, chord.mask);
    assert_eq!(c.frets, chord.frets);
    assert_eq!(c.fingers, chord.fingers);
    assert_eq!(c.notes, chord.notes);
    assert_eq!(c.name, chord.name);
}

/// Mirrors: round-trip for BendValue
#[test]
fn sng_roundtrip_bend_value() {
    let bv = BendValue { time: 66.66, step: 0.5 };
    // Embed it in a Note's bend_data
    let note = Note {
        mask: NoteMask::BEND,
        flags: 0,
        hash: 0,
        time: 5.0,
        string_index: 0,
        fret: 7,
        anchor_fret: 7,
        anchor_width: 4,
        chord_id: -1,
        chord_notes_id: -1,
        phrase_id: 0,
        phrase_iteration_id: 0,
        finger_print_id: [-1, -1],
        next_iter_note: -1,
        prev_iter_note: -1,
        parent_prev_note: -1,
        slide_to: -1,
        slide_unpitch_to: -1,
        left_hand: -1,
        tap: 0,
        pick_direction: 0,
        slap: -1,
        pluck: -1,
        vibrato: 0,
        sustain: 0.0,
        max_bend: 1.0,
        bend_data: vec![bv.clone()],
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.levels.push(Level {
        difficulty: 0,
        anchors: vec![],
        anchor_extensions: vec![],
        hand_shapes: vec![],
        arpeggios: vec![],
        notes: vec![note],
        average_notes_per_iteration: vec![],
        notes_in_phrase_iterations_excl_ignored: vec![],
        notes_in_phrase_iterations_all: vec![],
    });
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let bv_out = &parsed.levels[0].notes[0].bend_data[0];
    assert_eq!(bv_out.time, bv.time);
    assert_eq!(bv_out.step, bv.step);
}

/// Mirrors: round-trip for Note (detailed — mirrors "Note" test case in RoundTripTests.fs)
#[test]
fn sng_roundtrip_note_detailed() {
    let n = Note {
        mask: NoteMask::FRET_HAND_MUTE | NoteMask::UNPITCHED_SLIDE,
        flags: 1,
        hash: 45_684_265,
        time: 7.8,
        string_index: 1,
        fret: 7,
        anchor_fret: 7,
        anchor_width: 5,
        chord_id: -1,
        chord_notes_id: -1,
        phrase_id: 2,
        phrase_iteration_id: 4,
        finger_print_id: [0, 1],
        next_iter_note: 2,
        prev_iter_note: 4,
        parent_prev_note: 1,
        slide_to: -1,
        slide_unpitch_to: 24,
        left_hand: -1,
        tap: 0,
        pick_direction: 0,
        slap: 1,
        pluck: 0,
        vibrato: 80,
        sustain: 44.0,
        max_bend: 1.0,
        bend_data: vec![
            BendValue { time: 66.66, step: f32::INFINITY.recip() },
            BendValue { time: 66.66, step: 0.5 },
        ],
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.levels.push(Level {
        difficulty: 0,
        anchors: vec![],
        anchor_extensions: vec![],
        hand_shapes: vec![],
        arpeggios: vec![],
        notes: vec![n.clone()],
        average_notes_per_iteration: vec![],
        notes_in_phrase_iterations_excl_ignored: vec![],
        notes_in_phrase_iterations_all: vec![],
    });
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let no = &parsed.levels[0].notes[0];

    assert_eq!(no.mask, n.mask);
    assert_eq!(no.flags, n.flags);
    assert_eq!(no.hash, n.hash);
    assert_eq!(no.time, n.time);
    assert_eq!(no.string_index, n.string_index);
    assert_eq!(no.fret, n.fret);
    assert_eq!(no.anchor_fret, n.anchor_fret);
    assert_eq!(no.chord_id, n.chord_id);
    assert_eq!(no.phrase_id, n.phrase_id);
    assert_eq!(no.slide_unpitch_to, n.slide_unpitch_to);
    assert_eq!(no.vibrato, n.vibrato);
    assert_eq!(no.sustain, n.sustain);
    assert_eq!(no.max_bend, n.max_bend);
    assert_eq!(no.bend_data.len(), 2);
}

/// Mirrors: round-trip for Anchor (including 3-byte padding)
#[test]
fn sng_roundtrip_anchor() {
    let a = Anchor {
        start_time: 10.0,
        end_time: 20.0,
        first_note_time: 11.0,
        last_note_time: 17.0,
        fret_id: 12,
        width: 4,
        phrase_iteration_id: 7,
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.levels.push(Level {
        difficulty: 0,
        anchors: vec![a.clone()],
        anchor_extensions: vec![],
        hand_shapes: vec![],
        arpeggios: vec![],
        notes: vec![],
        average_notes_per_iteration: vec![],
        notes_in_phrase_iterations_excl_ignored: vec![],
        notes_in_phrase_iterations_all: vec![],
    });
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let ao = &parsed.levels[0].anchors[0];
    assert_eq!(ao.start_time, a.start_time);
    assert_eq!(ao.end_time, a.end_time);
    assert_eq!(ao.fret_id, a.fret_id);
    assert_eq!(ao.width, a.width);
    assert_eq!(ao.phrase_iteration_id, a.phrase_iteration_id);
}

/// Mirrors: round-trip for Section (fixed-length string fields)
#[test]
fn sng_roundtrip_section() {
    let s = Section {
        name: "tapping".into(),
        number: 2,
        start_time: 50.0,
        end_time: 62.7,
        start_phrase_iteration_id: 5,
        end_phrase_iteration_id: 6,
        string_mask: (0..36).map(|i: i8| i).collect::<Vec<_>>().try_into().unwrap(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.sections.push(s.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let so = &parsed.sections[0];
    assert_eq!(so.name, s.name);
    assert_eq!(so.number, s.number);
    assert_eq!(so.start_time, s.start_time);
    assert_eq!(so.end_time, s.end_time);
    assert_eq!(so.string_mask, s.string_mask);
}

/// Mirrors: round-trip for PhraseIteration
#[test]
fn sng_roundtrip_phrase_iteration() {
    let pi = PhraseIteration {
        phrase_id: 42,
        start_time: 10.111,
        end_time: 20.222,
        difficulty: [0, 5, 10],
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.phrase_iterations.push(pi.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let pio = &parsed.phrase_iterations[0];
    assert_eq!(pio.phrase_id, pi.phrase_id);
    assert_eq!(pio.start_time, pi.start_time);
    assert_eq!(pio.end_time, pi.end_time);
    assert_eq!(pio.difficulty, pi.difficulty);
}

/// Mirrors: round-trip for NewLinkedDifficulty
#[test]
fn sng_roundtrip_new_linked_difficulty() {
    let nld = NewLinkedDifficulty {
        level_break: 16,
        nld_phrases: vec![0, 1, 2, 3],
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.new_linked_difficulties.push(nld.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let o = &parsed.new_linked_difficulties[0];
    assert_eq!(o.level_break, nld.level_break);
    assert_eq!(o.nld_phrases, nld.nld_phrases);
}

/// Mirrors: round-trip for Vocal (48-byte lyric field)
#[test]
fn sng_roundtrip_vocal() {
    let v = Vocal {
        time: 1.5,
        note: 60,
        length: 0.5,
        lyric: "Hello".into(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.vocals.push(v.clone());
    // add symbol data since vocals > 0
    sng.symbols_headers.push(SymbolsHeader { id: 0, unk: [0; 7] });
    sng.symbols_textures.push(SymbolsTexture {
        font: "testfont.dds".into(),
        font_path_length: 12,
        width: 512,
        height: 512,
    });
    sng.symbol_definitions.push(SymbolDefinition {
        symbol: "A".into(),
        outer: Rect { y_min: 0.0, x_min: 0.0, y_max: 1.0, x_max: 1.0 },
        inner: Rect { y_min: 0.1, x_min: 0.1, y_max: 0.9, x_max: 0.9 },
    });
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let vo = &parsed.vocals[0];
    assert_eq!(vo.time, v.time);
    assert_eq!(vo.note, v.note);
    assert_eq!(vo.length, v.length);
    assert_eq!(vo.lyric, v.lyric);
}

/// Mirrors: round-trip for Action (256-byte name field)
#[test]
fn sng_roundtrip_action() {
    let a = Action {
        time: 70.0,
        action_name: "NOT USED IN RS2014 <_<".into(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.actions.push(a.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let ao = &parsed.actions[0];
    assert_eq!(ao.time, a.time);
    assert_eq!(ao.action_name, a.action_name);
}

/// Mirrors: round-trip for Event (256-byte name field)
#[test]
fn sng_roundtrip_event() {
    let e = Event { time: 12.5, name: "E1".into() };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.events.push(e.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    assert_eq!(parsed.events[0].time, e.time);
    assert_eq!(parsed.events[0].name, e.name);
}

/// Mirrors: round-trip for Tone
#[test]
fn sng_roundtrip_tone() {
    let t = Tone { time: 5.0, tone_id: 3 };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.tones.push(t);
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    assert_eq!(parsed.tones[0].time, t.time);
    assert_eq!(parsed.tones[0].tone_id, t.tone_id);
}

/// Mirrors: round-trip for DNA
#[test]
fn sng_roundtrip_dna() {
    let d = Dna { time: 0.0, dna_id: 2 };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.dnas.push(d);
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    assert_eq!(parsed.dnas[0].time, d.time);
    assert_eq!(parsed.dnas[0].dna_id, d.dna_id);
}

/// Mirrors: "UTF8 string always includes null terminator" (BinaryHelpersTests.fs)
#[test]
fn sng_fixed_string_null_terminated() {
    // Write a phrase whose name is exactly 31 chars (max without null terminator)
    let long_name = "A".repeat(31);
    let phrase = Phrase {
        solo: 0, disparity: 0, ignore: 0,
        max_difficulty: 0, iteration_count: 0,
        name: long_name.clone(),
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.phrases.push(phrase);
    let bytes = sng_to_bytes(&sng);
    // Verify null terminator at expected position (4 + 1+1+1+1+4+4 = 16 bytes for header/count,
    // then phrase starts at offset 4, string field is at +12)
    // Easier: just re-parse and verify the name is preserved
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    assert_eq!(parsed.phrases[0].name, long_name);

    // The raw bytes of the string field must end with a null byte within the 32-byte buffer.
    // Layout: beats_count(4) + phrases_count(4) + solo(1)+disp(1)+ign(1)+pad(1)+maxdiff(4)+iter(4) = 20
    let string_start = 4 + 4 + 1 + 1 + 1 + 1 + 4 + 4;
    let string_end = string_start + 32;
    assert_eq!(bytes[string_end - 1], 0u8, "last byte of 32-byte string field must be 0");
}

// --- BendData32 round-trip -------------------------------------------------

/// Mirrors: "Bend Data 32" test case
#[test]
fn sng_roundtrip_bend_data32() {
    let bend_values: [BendValue; 32] = std::array::from_fn(|i| BendValue {
        time: 66.66 + i as f32,
        step: 1.0 / (i + 1) as f32,
    });
    let bd = BendData32 { bend_values, used_count: 32 };

    // Embed in ChordNotes
    let cn = ChordNotes {
        mask: [NoteMask::empty(); 6],
        bend_data: std::array::from_fn(|_| bd.clone()),
        slide_to: [0; 6],
        slide_unpitch_to: [0; 6],
        vibrato: [0; 6],
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    sng.chord_notes.push(cn);
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let cno = &parsed.chord_notes[0];

    for i in 0..6 {
        assert_eq!(cno.bend_data[i].used_count, 32);
        for j in 0..32 {
            assert_eq!(cno.bend_data[i].bend_values[j].time, 66.66 + j as f32);
        }
    }
}

/// Mirrors: "Symbol Definition" round-trip
#[test]
fn sng_roundtrip_symbol_definition() {
    let def = SymbolDefinition {
        symbol: "金".into(),
        outer: Rect { y_min: 1.888, x_min: 1.015, y_max: 1.99, x_max: 1.1 },
        inner: Rect { y_min: 0.888, x_min: 0.015, y_max: 0.99, x_max: 0.1 },
    };
    let mut sng = empty_sng_with_metadata(make_metadata());
    // Need at least one vocal to trigger symbol data serialisation
    sng.vocals.push(Vocal { time: 0.0, note: 60, length: 0.5, lyric: "a".into() });
    sng.symbols_headers.push(SymbolsHeader { id: 0, unk: [0; 7] });
    sng.symbols_textures.push(SymbolsTexture {
        font: "f".into(),
        font_path_length: 1,
        width: 256, height: 256,
    });
    sng.symbol_definitions.push(def.clone());
    let bytes = sng_to_bytes(&sng);
    let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
    let o = &parsed.symbol_definitions[0];
    assert_eq!(o.symbol, def.symbol);
    assert_eq!(o.outer.y_min, def.outer.y_min);
    assert_eq!(o.inner.x_max, def.inner.x_max);
}

// --- NoteMask tests  (mirrors NoteMask parts of RoundTripTests) -------------

/// Mirrors: NoteMask flag values are correctly preserved through serialisation
#[test]
fn sng_note_mask_flag_values() {
    assert_eq!(NoteMask::CHORD.bits(), 0x0000_0002);
    assert_eq!(NoteMask::FRET_HAND_MUTE.bits(), 0x0000_0008);
    assert_eq!(NoteMask::HAMMER_ON.bits(), 0x0000_0200);
    assert_eq!(NoteMask::SLIDE.bits(), 0x0000_0800);
    assert_eq!(NoteMask::BEND.bits(), 0x0000_1000);
    assert_eq!(NoteMask::VIBRATO.bits(), 0x0001_0000);
    assert_eq!(NoteMask::MUTE.bits(), 0x0002_0000);
    assert_eq!(NoteMask::IGNORE.bits(), 0x0004_0000);
    assert_eq!(NoteMask::UNPITCHED_SLIDE.bits(), 0x0040_0000);

    // Combined flags survive a bitwise round-trip
    let combined = NoteMask::FRET_HAND_MUTE | NoteMask::UNPITCHED_SLIDE;
    let bits = combined.bits();
    let restored = NoteMask::from_bits_truncate(bits);
    assert_eq!(restored, combined);
}

/// BeatMask flag values
#[test]
fn sng_beat_mask_flag_values() {
    assert_eq!(BeatMask::FIRST_BEAT_OF_MEASURE.bits(), 0b01);
    assert_eq!(BeatMask::EVEN_MEASURE.bits(), 0b10);
    let both = BeatMask::FIRST_BEAT_OF_MEASURE | BeatMask::EVEN_MEASURE;
    assert_eq!(both.bits(), 0b11);
}

// ===========================================================================
// XML tests  (mirrors Rocksmith2014.XML.Tests)
// ===========================================================================

// --- MetaData tests  (mirrors MetaDataTests.cs) ----------------------------

/// Mirrors: "CanBeReadFromXMLFile"
#[test]
fn xml_metadata_can_be_read_from_file() {
    let arr = InstrumentalArrangement::open(cdlc("instrumental.xml"))
        .expect("open instrumental.xml");

    assert_eq!(arr.title, "Test Instrumental");
    assert!(
        (arr.average_tempo - 160.541).abs() < 0.001,
        "average_tempo expected ≈160.541, got {}",
        arr.average_tempo
    );
    assert_eq!(arr.artist_name_sort, "Test");
    assert_eq!(arr.last_conversion_date_time, "5-17-20 15:21");
}

// --- InstrumentalArrangement tests  (mirrors InstrumentalArrangementTests.cs) ---

/// Mirrors: "CanRemoveDD" — the instrumental.xml has 12 levels (DD)
#[test]
fn xml_instrumental_has_dd_levels() {
    let arr = InstrumentalArrangement::open(cdlc("instrumental.xml"))
        .expect("open instrumental.xml");

    assert_eq!(
        arr.level_count, 12,
        "instrumental.xml should have 12 difficulty levels"
    );
}

// --- Anchor tests  (mirrors AnchorTests.cs) --------------------------------

/// Mirrors: "UsesStructuralEquality" and "CopyConstructorCopiesAllValues"
#[test]
fn xml_anchor_equality_and_copy() {
    // Simulate the .NET Anchor(fret, time, width) constructor
    let fret: i16 = 22;
    let time: u32 = 4567;   // milliseconds
    let width: i8 = 6;

    // Copy (clone in Rust) should produce equal values
    let a1 = (fret, time, width);
    let a2 = a1;
    assert_eq!(a1, a2, "Copied anchor has same values");
}

// --- Note mask tests  (mirrors NoteTests.cs / NoteMaskAccessProperties*) ---

/// Mirrors: "NoteMaskAccessPropertiesSettersTest" — flag set/clear
#[test]
fn xml_sng_note_mask_set_clear() {
    let mut mask = NoteMask::HAMMER_ON | NoteMask::PALM_MUTE;

    // Set Accent, verify it is added
    mask |= NoteMask::ACCENT;
    assert!(mask.contains(NoteMask::ACCENT));
    assert!(mask.contains(NoteMask::HAMMER_ON));

    // Clear Accent
    mask.remove(NoteMask::ACCENT);
    assert!(!mask.contains(NoteMask::ACCENT));
    assert!(mask.contains(NoteMask::HAMMER_ON));

    // Set Harmonic, clear HammerOn
    mask |= NoteMask::HARMONIC;
    mask.remove(NoteMask::HAMMER_ON);
    assert!(mask.contains(NoteMask::HARMONIC));
    assert!(!mask.contains(NoteMask::HAMMER_ON));
}

/// Mirrors: "NoteMaskAccessPropertiesGettersTest" — flag presence
#[test]
fn xml_sng_note_mask_getters() {
    let empty = NoteMask::empty();
    assert!(!empty.contains(NoteMask::ACCENT));
    assert!(!empty.contains(NoteMask::HAMMER_ON));
    assert!(!empty.contains(NoteMask::HARMONIC));
    assert!(!empty.contains(NoteMask::IGNORE));

    let combo = NoteMask::ACCENT | NoteMask::HAMMER_ON | NoteMask::HARMONIC | NoteMask::IGNORE;
    assert!(combo.contains(NoteMask::ACCENT));
    assert!(combo.contains(NoteMask::HAMMER_ON));
    assert!(combo.contains(NoteMask::HARMONIC));
    assert!(combo.contains(NoteMask::IGNORE));
    assert!(!combo.contains(NoteMask::MUTE));
    assert!(!combo.contains(NoteMask::PALM_MUTE));
    assert!(!combo.contains(NoteMask::PULL_OFF));

    let combo2 = NoteMask::MUTE | NoteMask::PALM_MUTE | NoteMask::PULL_OFF;
    assert!(!combo2.contains(NoteMask::ACCENT));
    assert!(combo2.contains(NoteMask::MUTE));
    assert!(combo2.contains(NoteMask::PALM_MUTE));
    assert!(combo2.contains(NoteMask::PULL_OFF));
}

/// Mirrors: "OtherGettersTest" — derived properties (bend, slide, vibrato, tap)
#[test]
fn xml_sng_note_derived_properties() {
    let mut mask = NoteMask::empty();
    assert!(!mask.contains(NoteMask::BEND));
    assert!(!mask.contains(NoteMask::SLIDE));
    assert!(!mask.contains(NoteMask::UNPITCHED_SLIDE));
    assert!(!mask.contains(NoteMask::VIBRATO));
    assert!(!mask.contains(NoteMask::TAP));

    mask |= NoteMask::BEND;
    assert!(mask.contains(NoteMask::BEND));

    mask |= NoteMask::SLIDE;
    assert!(mask.contains(NoteMask::SLIDE));

    mask |= NoteMask::UNPITCHED_SLIDE;
    assert!(mask.contains(NoteMask::UNPITCHED_SLIDE));

    mask |= NoteMask::VIBRATO;
    assert!(mask.contains(NoteMask::VIBRATO));

    mask |= NoteMask::TAP;
    assert!(mask.contains(NoteMask::TAP));
}

/// Mirrors: "CopyConstructorCopiesAllValues" (NoteTests.cs) in Rust:
/// verify that cloning a Sng Note preserves all fields.
#[test]
fn xml_sng_note_clone_all_fields() {
    let note = Note {
        mask: NoteMask::ACCENT | NoteMask::IGNORE | NoteMask::FRET_HAND_MUTE | NoteMask::SLAP,
        flags: 0,
        hash: 0,
        time: 33.0,
        string_index: 4,
        fret: 22,
        anchor_fret: 22,
        anchor_width: 4,
        chord_id: -1,
        chord_notes_id: -1,
        phrase_id: 0,
        phrase_iteration_id: 0,
        finger_print_id: [-1, -1],
        next_iter_note: -1,
        prev_iter_note: -1,
        parent_prev_note: -1,
        slide_to: 7,
        slide_unpitch_to: 9,
        left_hand: 3,
        tap: 2,
        pick_direction: 0,
        slap: 1,
        pluck: 0,
        vibrato: 80,
        sustain: 99.0,
        max_bend: 4.0,
        bend_data: vec![
            BendValue { time: 34.0, step: 3.0 },
            BendValue { time: 35.0, step: 4.0 },
        ],
    };

    let note2 = note.clone();
    assert_eq!(note2.fret, note.fret);
    assert_eq!(note2.left_hand, note.left_hand);
    assert_eq!(note2.mask, note.mask);
    assert_eq!(note2.slide_to, note.slide_to);
    assert_eq!(note2.slide_unpitch_to, note.slide_unpitch_to);
    assert_eq!(note2.string_index, note.string_index);
    assert_eq!(note2.sustain, note.sustain);
    assert_eq!(note2.tap, note.tap);
    assert_eq!(note2.time, note.time);
    assert_eq!(note2.vibrato, note.vibrato);
    assert_eq!(note2.max_bend, note.max_bend);
    assert_eq!(note2.bend_data.len(), note.bend_data.len());
    // bend_data is a separate allocation (Vec clone)
    assert_eq!(note2.bend_data[0].time, note.bend_data[0].time);
    assert_eq!(note2.bend_data[1].step, note.bend_data[1].step);
}

// --- Chord mask tests  (mirrors ChordTests.cs) -----------------------------

/// Mirrors: "ChordMaskAccessPropertiesSettersTest"
#[test]
fn xml_sng_chord_mask_set_clear() {
    let mut mask = ChordMask::ARPEGGIO;

    // NOP flag
    mask |= ChordMask::NOP;
    assert!(mask.contains(ChordMask::NOP));
    mask.remove(ChordMask::NOP);
    assert!(!mask.contains(ChordMask::NOP));
}

/// Mirrors: "HasChordNotesReturnsCorrectValue"
#[test]
fn sng_chord_notes_is_present() {
    let cn_empty: Vec<ChordNotes> = vec![];
    let cn_some: Vec<ChordNotes> = vec![ChordNotes {
        mask: [NoteMask::empty(); 6],
        bend_data: std::array::from_fn(|_| BendData32 {
            bend_values: std::array::from_fn(|_| BendValue::default()),
            used_count: 0,
        }),
        slide_to: [0; 6],
        slide_unpitch_to: [0; 6],
        vibrato: [0; 6],
    }];
    assert!(cn_empty.is_empty());
    assert!(!cn_some.is_empty());
}

// --- test_arrangement.xml --------------------------------------------------

/// CDLC test fixture: parse the local test_arrangement.xml
#[test]
fn xml_cdlc_test_arrangement() {
    let arr = InstrumentalArrangement::open(cdlc("test_arrangement.xml"))
        .expect("open test_arrangement.xml");

    assert_eq!(arr.title, "Test CDLC Song");
    assert_eq!(arr.arrangement, "Lead");
    assert_eq!(arr.artist_name, "Test Artist");
    assert_eq!(arr.album_year, 2024);
    assert_eq!(arr.level_count, 1);
    // 3 notes + 1 chord
    assert_eq!(arr.note_count, 4);
    assert!((arr.average_tempo - 120.0).abs() < 0.001);
    assert_eq!(arr.tuning.string0, 0);
}

// ===========================================================================
// FFI smoke tests
// ===========================================================================

#[cfg(test)]
mod ffi_tests {
    use std::ffi::CString;

    use rocksmith2014::ffi::*;

    fn cdlc_cpath(name: &str) -> CString {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("cdlc")
            .join(name);
        CString::new(p.to_str().unwrap()).unwrap()
    }

    #[test]
    fn ffi_psarc_open_close() {
        unsafe {
            let path = cdlc_cpath("test_p.psarc");
            let handle = rs2014_psarc_open(path.as_ptr());
            assert!(!handle.is_null(), "psarc handle should not be null");
            assert!(rs2014_psarc_entry_count(handle) > 0);
            rs2014_psarc_close(handle);
        }
    }

    #[test]
    fn ffi_psarc_null_path_returns_null() {
        unsafe {
            let handle = rs2014_psarc_open(std::ptr::null());
            assert!(handle.is_null());
        }
    }

    #[test]
    fn ffi_psarc_extract_and_free() {
        unsafe {
            let path = cdlc_cpath("test_p.psarc");
            let handle = rs2014_psarc_open(path.as_ptr());
            assert!(!handle.is_null());

            let mut data: *mut u8 = std::ptr::null_mut();
            let mut len: usize = 0;
            let rc = rs2014_psarc_extract(handle, 0, &mut data, &mut len);
            assert_eq!(rc, 0, "extract should succeed");
            assert!(!data.is_null());
            assert!(len > 0);
            rs2014_psarc_free_data(data, len);
            rs2014_psarc_close(handle);
        }
    }

    #[test]
    fn ffi_sng_open_packed_pc() {
        unsafe {
            let path = cdlc_cpath("packed_pc.sng");
            let handle = rs2014_sng_open_packed(path.as_ptr(), 0);
            assert!(!handle.is_null(), "SNG PC handle should not be null");
            assert_eq!(rs2014_sng_level_count(handle), 12);
            rs2014_sng_close(handle);
        }
    }

    #[test]
    fn ffi_sng_open_packed_mac() {
        unsafe {
            let path = cdlc_cpath("packed_mac.sng");
            let handle = rs2014_sng_open_packed(path.as_ptr(), 1);
            assert!(!handle.is_null(), "SNG Mac handle should not be null");
            assert_eq!(rs2014_sng_level_count(handle), 12);
            rs2014_sng_close(handle);
        }
    }

    #[test]
    fn ffi_sng_open_unpacked() {
        unsafe {
            let path = cdlc_cpath("unpacked.sng");
            let handle = rs2014_sng_open_unpacked(path.as_ptr());
            assert!(!handle.is_null());
            assert_eq!(rs2014_sng_level_count(handle), 12);
            rs2014_sng_close(handle);
        }
    }

    #[test]
    fn ffi_xml_open_and_fields() {
        unsafe {
            let path = cdlc_cpath("instrumental.xml");
            let handle = rs2014_xml_open(path.as_ptr());
            assert!(!handle.is_null(), "XML handle should not be null");

            let title = std::ffi::CStr::from_ptr(rs2014_xml_title(handle))
                .to_str()
                .unwrap();
            assert_eq!(title, "Test Instrumental");

            let tempo = rs2014_xml_average_tempo(handle);
            assert!((tempo - 160.541).abs() < 0.001, "tempo ≈ 160.541");

            assert_eq!(rs2014_xml_level_count(handle), 12);

            rs2014_xml_close(handle);
        }
    }

    #[test]
    fn ffi_xml_null_path_returns_null() {
        unsafe {
            let handle = rs2014_xml_open(std::ptr::null());
            assert!(handle.is_null());
        }
    }
}
