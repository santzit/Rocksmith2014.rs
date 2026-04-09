//! Mirrors RoundTripTests.fs from Rocksmith2014.NET tests.

use rocksmith2014_sng::{
    Action, Anchor, AnchorExtension, Beat, BeatMask, BendData32, BendValue, Chord, ChordNotes,
    Event, FingerPrint, Level, MetaData, NewLinkedDifficulty, Note, NoteMask, Phrase,
    PhraseExtraInfo, PhraseIteration, Platform, Rect, Section, Sng, SymbolDefinition,
    SymbolsHeader, SymbolsTexture, Tone, Vocal, DNA,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sng_roundtrip(sng: Sng) -> Sng {
    let bytes = sng.write().unwrap();
    Sng::read(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// Beat
// ---------------------------------------------------------------------------

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
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.beats.len(), 1);
    assert_eq!(sng2.beats[0].time, 1.0);
    assert_eq!(sng2.beats[0].mask, BeatMask::FIRST_BEAT_OF_MEASURE);
}

// ---------------------------------------------------------------------------
// Phrase
// ---------------------------------------------------------------------------

#[test]
fn test_phrase_roundtrip() {
    let mut name = [0u8; 32];
    name[..5].copy_from_slice(b"verse");
    let mut sng = Sng::default();
    sng.phrases.push(Phrase {
        solo: 0,
        disparity: 1,
        ignore: 0,
        max_difficulty: 5,
        iteration_count: 3,
        name,
    });
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.phrases.len(), 1);
    assert_eq!(sng2.phrases[0].max_difficulty, 5);
    assert_eq!(sng2.phrases[0].name[..5], *b"verse");
}

// ---------------------------------------------------------------------------
// Chord
// ---------------------------------------------------------------------------

#[test]
fn test_chord_roundtrip() {
    let mut sng = Sng::default();
    sng.chords.push(Chord {
        mask: rocksmith2014_sng::ChordMask::empty(),
        frets: [0, 2, 2, 1, 0, 0],
        fingers: [0, 1, 2, 3, 0, 0],
        notes: [-1; 6],
        name: [0u8; 32],
    });
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.chords.len(), 1);
    assert_eq!(sng2.chords[0].frets, [0, 2, 2, 1, 0, 0]);
    assert_eq!(sng2.chords[0].fingers, [0, 1, 2, 3, 0, 0]);
}

// ---------------------------------------------------------------------------
// BendValue (tested via Note)
// ---------------------------------------------------------------------------

#[test]
fn test_bend_value_roundtrip() {
    let bv = BendValue {
        time: 66.66,
        step: 1.0,
        unused: 0,
    };
    let note = Note {
        mask: NoteMask::SINGLE,
        time: 1.0,
        bend_data: vec![bv],
        ..Note::default()
    };
    let mut sng = Sng::default();
    sng.levels.push(Level {
        notes: vec![note],
        ..Level::default()
    });
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.levels[0].notes[0].bend_data.len(), 1);
    assert_eq!(sng2.levels[0].notes[0].bend_data[0].time, 66.66);
    assert_eq!(sng2.levels[0].notes[0].bend_data[0].step, 1.0);
}

// ---------------------------------------------------------------------------
// BendData32 (tested via ChordNotes)
// ---------------------------------------------------------------------------

#[test]
fn test_bend_data32_roundtrip() {
    let mut bend_values: [BendValue; 32] = std::array::from_fn(|_| BendValue::default());
    for (i, bv) in bend_values.iter_mut().enumerate() {
        bv.time = 66.66 + i as f32;
        bv.step = 1.0 / (i + 1) as f32;
    }
    let mut chord_notes = ChordNotes::default();
    chord_notes.bend_data[0] = BendData32 {
        bend_values,
        used_count: 32,
    };
    let mut sng = Sng::default();
    sng.chord_notes.push(chord_notes);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.chord_notes.len(), 1);
    assert_eq!(sng2.chord_notes[0].bend_data[0].used_count, 32);
    assert_eq!(sng2.chord_notes[0].bend_data[0].bend_values[0].time, 66.66);
}

// ---------------------------------------------------------------------------
// ChordNotes
// ---------------------------------------------------------------------------

#[test]
fn test_chord_notes_roundtrip() {
    let mut cn = ChordNotes::default();
    cn.mask = [1, 2, 3, 4, 5, 6];
    cn.slide_to = [0, 1, 2, 3, 4, 5];
    cn.slide_unpitch_to = [-1; 6];
    cn.vibrato = [0, 80, 0, 0, 0, 0];
    let mut sng = Sng::default();
    sng.chord_notes.push(cn);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.chord_notes.len(), 1);
    assert_eq!(sng2.chord_notes[0].mask, [1, 2, 3, 4, 5, 6]);
    assert_eq!(sng2.chord_notes[0].slide_to, [0, 1, 2, 3, 4, 5]);
    assert_eq!(sng2.chord_notes[0].vibrato, [0, 80, 0, 0, 0, 0]);
}

// ---------------------------------------------------------------------------
// Vocal (with symbols, because symbol fields are gated on non-empty vocals)
// ---------------------------------------------------------------------------

#[test]
fn test_vocal_roundtrip() {
    let mut lyric = [0u8; 48];
    lyric[..5].copy_from_slice(b"hello");
    let mut sng = Sng::default();
    sng.vocals.push(Vocal {
        time: 1.5,
        note: 60,
        length: 0.8,
        lyric,
    });
    // Symbol fields required when vocals is non-empty
    sng.symbols_headers.push(SymbolsHeader::default());
    sng.symbols_textures.push(SymbolsTexture::default());
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.vocals.len(), 1);
    assert_eq!(sng2.vocals[0].time, 1.5);
    assert_eq!(sng2.vocals[0].note, 60);
    assert_eq!(sng2.vocals[0].lyric[..5], *b"hello");
}

// ---------------------------------------------------------------------------
// SymbolsHeader
// ---------------------------------------------------------------------------

#[test]
fn test_symbols_header_roundtrip() {
    let header = SymbolsHeader {
        id: 42,
        unk2: 1,
        unk3: 2,
        unk4: 3,
        unk5: 4,
        unk6: 5,
        unk7: 6,
        unk8: 7,
    };
    let mut sng = Sng::default();
    sng.vocals.push(Vocal::default()); // needed to enable symbol write
    sng.symbols_headers.push(header);
    sng.symbols_textures.push(SymbolsTexture::default());
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.symbols_headers.len(), 1);
    assert_eq!(sng2.symbols_headers[0].id, 42);
}

// ---------------------------------------------------------------------------
// SymbolsTexture
// ---------------------------------------------------------------------------

#[test]
fn test_symbols_texture_roundtrip() {
    let mut font = [0u8; 128];
    font[..4].copy_from_slice(b"test");
    let tex = SymbolsTexture {
        font,
        font_path_length: 4,
        width: 512,
        height: 256,
    };
    let mut sng = Sng::default();
    sng.vocals.push(Vocal::default());
    sng.symbols_headers.push(SymbolsHeader::default());
    sng.symbols_textures.push(tex);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.symbols_textures.len(), 1);
    assert_eq!(sng2.symbols_textures[0].width, 512);
    assert_eq!(sng2.symbols_textures[0].height, 256);
    assert_eq!(sng2.symbols_textures[0].font_path_length, 4);
}

// ---------------------------------------------------------------------------
// Rect (tested via SymbolDefinition)
// ---------------------------------------------------------------------------

#[test]
fn test_rect_roundtrip() {
    let outer = Rect {
        ymin: 1.888,
        xmin: 1.015,
        ymax: 1.99,
        xmax: 1.1,
    };
    let inner = Rect {
        ymin: 0.888,
        xmin: 0.015,
        ymax: 0.99,
        xmax: 0.1,
    };
    let mut symbol_bytes = [0u8; 12];
    symbol_bytes[..3].copy_from_slice("金".as_bytes()); // 3-byte UTF-8
    let def = SymbolDefinition {
        symbol: symbol_bytes,
        outer,
        inner,
    };
    let mut sng = Sng::default();
    sng.vocals.push(Vocal::default());
    sng.symbols_headers.push(SymbolsHeader::default());
    sng.symbols_textures.push(SymbolsTexture::default());
    sng.symbol_definitions.push(def);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.symbol_definitions.len(), 1);
    assert_eq!(sng2.symbol_definitions[0].outer.ymin, 1.888);
    assert_eq!(sng2.symbol_definitions[0].inner.xmin, 0.015);
}

// ---------------------------------------------------------------------------
// PhraseIteration
// ---------------------------------------------------------------------------

#[test]
fn test_phrase_iteration_roundtrip() {
    let pi = PhraseIteration {
        phrase_id: 42,
        start_time: 10.111,
        end_time: 20.222,
        difficulty: [0, 1, 2],
    };
    let mut sng = Sng::default();
    sng.phrase_iterations.push(pi);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.phrase_iterations.len(), 1);
    assert_eq!(sng2.phrase_iterations[0].phrase_id, 42);
    assert_eq!(sng2.phrase_iterations[0].start_time, 10.111);
    assert_eq!(sng2.phrase_iterations[0].difficulty, [0, 1, 2]);
}

// ---------------------------------------------------------------------------
// PhraseExtraInfo
// ---------------------------------------------------------------------------

#[test]
fn test_phrase_extra_info_roundtrip() {
    let info = PhraseExtraInfo {
        phrase_id: 3,
        difficulty: 7,
        empty: 0,
        level_jump: 1,
        redundant: -1,
    };
    let mut sng = Sng::default();
    sng.phrase_extra_info.push(info);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.phrase_extra_info.len(), 1);
    assert_eq!(sng2.phrase_extra_info[0].phrase_id, 3);
    assert_eq!(sng2.phrase_extra_info[0].level_jump, 1);
}

// ---------------------------------------------------------------------------
// NewLinkedDifficulty
// ---------------------------------------------------------------------------

#[test]
fn test_new_linked_difficulty_roundtrip() {
    let nld = NewLinkedDifficulty {
        level_break: 16,
        nld_phrases: vec![0, 1, 2, 3],
    };
    let mut sng = Sng::default();
    sng.new_linked_difficulties.push(nld);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.new_linked_difficulties.len(), 1);
    assert_eq!(sng2.new_linked_difficulties[0].level_break, 16);
    assert_eq!(
        sng2.new_linked_difficulties[0].nld_phrases,
        vec![0, 1, 2, 3]
    );
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

#[test]
fn test_action_roundtrip() {
    let mut action_name = [0u8; 256];
    let name = b"NOT USED IN RS2014";
    action_name[..name.len()].copy_from_slice(name);
    let action = Action {
        time: 70.0,
        action_name,
    };
    let mut sng = Sng::default();
    sng.actions.push(action);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.actions.len(), 1);
    assert_eq!(sng2.actions[0].time, 70.0);
    assert_eq!(&sng2.actions[0].action_name[..name.len()], name.as_slice());
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[test]
fn test_event_roundtrip() {
    let mut name = [0u8; 256];
    name[..5].copy_from_slice(b"E0+E1");
    let event = Event { time: 15.5, name };
    let mut sng = Sng::default();
    sng.events.push(event);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.events.len(), 1);
    assert_eq!(sng2.events[0].time, 15.5);
    assert_eq!(&sng2.events[0].name[..5], b"E0+E1");
}

// ---------------------------------------------------------------------------
// Tone (SNG tone change, not manifest Tone)
// ---------------------------------------------------------------------------

#[test]
fn test_tone_roundtrip() {
    let mut sng = Sng::default();
    sng.tones.push(Tone {
        time: 30.0,
        tone_id: 2,
    });
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.tones.len(), 1);
    assert_eq!(sng2.tones[0].time, 30.0);
    assert_eq!(sng2.tones[0].tone_id, 2);
}

// ---------------------------------------------------------------------------
// DNA
// ---------------------------------------------------------------------------

#[test]
fn test_dna_roundtrip() {
    let mut sng = Sng::default();
    sng.dnas.push(DNA {
        time: 5.0,
        dna_id: 1,
    });
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.dnas.len(), 1);
    assert_eq!(sng2.dnas[0].dna_id, 1);
}

// ---------------------------------------------------------------------------
// Section
// ---------------------------------------------------------------------------

#[test]
fn test_section_roundtrip() {
    let mut name = [0u8; 32];
    name[..7].copy_from_slice(b"tapping");
    let section = Section {
        name,
        number: 2,
        start_time: 50.0,
        end_time: 62.7,
        start_phrase_iteration_id: 5,
        end_phrase_iteration_id: 6,
        string_mask: [1i8; 36],
    };
    let mut sng = Sng::default();
    sng.sections.push(section);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.sections.len(), 1);
    assert_eq!(sng2.sections[0].number, 2);
    assert_eq!(sng2.sections[0].start_time, 50.0);
    assert_eq!(&sng2.sections[0].name[..7], b"tapping");
}

// ---------------------------------------------------------------------------
// Anchor
// ---------------------------------------------------------------------------

#[test]
fn test_anchor_roundtrip() {
    let anchor = Anchor {
        start_time: 10.0,
        end_time: 20.0,
        first_note_time: 11.0,
        last_note_time: 17.0,
        fret_id: 12,
        width: 4,
        phrase_iteration_id: 7,
    };
    let level = Level {
        anchors: vec![anchor],
        ..Level::default()
    };
    let mut sng = Sng::default();
    sng.levels.push(level);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.levels[0].anchors[0].fret_id, 12);
    assert_eq!(sng2.levels[0].anchors[0].width, 4);
}

// ---------------------------------------------------------------------------
// AnchorExtension
// ---------------------------------------------------------------------------

#[test]
fn test_anchor_extension_roundtrip() {
    let level = Level {
        anchor_extensions: vec![AnchorExtension {
            beat_time: 5.5,
            fret_id: 7,
        }],
        ..Level::default()
    };
    let mut sng = Sng::default();
    sng.levels.push(level);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.levels[0].anchor_extensions[0].beat_time, 5.5);
    assert_eq!(sng2.levels[0].anchor_extensions[0].fret_id, 7);
}

// ---------------------------------------------------------------------------
// FingerPrint
// ---------------------------------------------------------------------------

#[test]
fn test_finger_print_roundtrip() {
    let fp = FingerPrint {
        chord_id: 3,
        start_time: 1.0,
        end_time: 2.0,
        first_note_time: 1.1,
        last_note_time: 1.9,
    };
    let level = Level {
        hand_shapes: vec![fp],
        ..Level::default()
    };
    let mut sng = Sng::default();
    sng.levels.push(level);
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.levels[0].hand_shapes[0].chord_id, 3);
}

// ---------------------------------------------------------------------------
// Note (detailed)
// ---------------------------------------------------------------------------

#[test]
fn test_note_roundtrip() {
    let note = Note {
        mask: NoteMask::FRET_HAND_MUTE | NoteMask::UNPITCHED_SLIDE,
        flags: 1,
        hash: 45684265,
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
        bend_data: vec![BendValue {
            time: 66.66,
            step: 1.0,
            unused: 0,
        }],
    };
    let mut sng = Sng::default();
    sng.levels.push(Level {
        notes: vec![note],
        ..Level::default()
    });
    let sng2 = sng_roundtrip(sng);
    let n = &sng2.levels[0].notes[0];
    assert_eq!(n.fret, 7);
    assert_eq!(n.vibrato, 80);
    assert_eq!(n.slide_unpitch_to, 24);
    assert_eq!(n.bend_data.len(), 1);
    assert_eq!(n.bend_data[0].time, 66.66);
}

// ---------------------------------------------------------------------------
// MetaData
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_roundtrip() {
    let mut dt = [0u8; 32];
    let s = b"6-11-18 18:36";
    dt[..s.len()].copy_from_slice(s);
    let metadata = MetaData {
        max_score: 100000.0,
        max_notes_and_chords: 456.0,
        max_notes_and_chords_real: 452.0,
        points_per_note: 100.0,
        first_beat_length: 88.0,
        start_time: 10.0,
        capo_fret_id: -1,
        last_conversion_date_time: dt,
        part: 1,
        song_length: 520.0,
        tuning: vec![0, 0, 0, 0, 0, 0],
        first_note_time: 15.0,
        max_difficulty: 22,
    };
    let mut sng = Sng::default();
    sng.metadata = metadata;
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.metadata.max_score, 100000.0);
    assert_eq!(sng2.metadata.capo_fret_id, -1);
    assert_eq!(sng2.metadata.part, 1);
    assert_eq!(sng2.metadata.max_difficulty, 22);
    assert_eq!(&sng2.metadata.last_conversion_date_time[..s.len()], s);
}

// ---------------------------------------------------------------------------
// Level round-trip (existing)
// ---------------------------------------------------------------------------

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
    let sng2 = sng_roundtrip(sng);
    assert_eq!(sng2.levels[0].anchors[0].fret_id, 1);
}

#[test]
fn test_empty_sng_roundtrip() {
    let sng = Sng::default();
    let sng2 = sng_roundtrip(sng);
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
