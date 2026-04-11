//! XML Objects → SNG Objects conversion tests.
//!
//! Mirrors `XmlObjectsToSngTests.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{
    flag_on_anchor_change, make_beat_converter, xml_convert_anchor, xml_convert_bend_value,
    xml_convert_chord_template, xml_convert_event, xml_convert_handshape, xml_convert_level,
    xml_convert_new_linked_difficulty, xml_convert_phrase, xml_convert_phrase_iteration,
    xml_convert_section, xml_convert_tone, xml_convert_vocal, xml_create_dnas,
    xml_create_meta_data, AccuData, NoteConverter, XmlEntity,
};
use rocksmith2014_xml::{
    Anchor, ArrangementEvent, BendValue, ChordTemplate, Ebeat, HandShape, InstrumentalArrangement,
    Level, NewLinkedDiff, Note, PhraseIteration, Section, ToneChange,
};
use rocksmith2014_xml::{Chord, Phrase as XmlPhrase};

fn create_test_arr() -> InstrumentalArrangement {
    let mut arr = InstrumentalArrangement::default();
    arr.meta.song_length = 4_784_455;
    arr.meta.start_beat = 1000;

    let f1 = [1i8; 6];
    let f2 = [1i8, 1, -1, -1, -1, -1];
    arr.chord_templates.push(ChordTemplate {
        name: "A".into(),
        display_name: "A".into(),
        fingers: f1,
        frets: f1,
    });
    arr.chord_templates.push(ChordTemplate {
        name: "A".into(),
        display_name: "A-arp".into(),
        fingers: f2,
        frets: f2,
    });

    arr.phrase_iterations.push(PhraseIteration {
        time: 1000,
        phrase_id: 1,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 2000,
        phrase_id: 1,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 3000,
        phrase_id: 77,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 7_554_100,
        phrase_id: 2,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 7_555_000,
        phrase_id: 3,
        ..Default::default()
    });

    arr.meta.tuning.strings = [-1, 2, 4, -5, 3, -2];

    arr.sections.push(Section {
        name: "1".into(),
        start_time: 1000,
        ..Default::default()
    });
    arr.sections.push(Section {
        name: "2".into(),
        start_time: 4000,
        ..Default::default()
    });
    arr.sections.push(Section {
        name: "3".into(),
        start_time: 8_000_000,
        ..Default::default()
    });

    arr.events.push(ArrangementEvent {
        time: 1000,
        code: "e0".into(),
    });
    arr.events.push(ArrangementEvent {
        time: 2500,
        code: "dna_none".into(),
    });
    arr.events.push(ArrangementEvent {
        time: 3500,
        code: "dna_solo".into(),
    });
    arr.events.push(ArrangementEvent {
        time: 4500,
        code: "dna_chord".into(),
    });
    arr.events.push(ArrangementEvent {
        time: 5500,
        code: "dna_riff".into(),
    });

    arr.ebeats.push(Ebeat {
        time: 1000,
        measure: 0,
    });
    arr.ebeats.push(Ebeat {
        time: 1200,
        measure: -1,
    });

    let mut lvl = Level::default();
    lvl.anchors.push(Anchor {
        time: 1000,
        fret: 8,
        width: 4,
        ..Default::default()
    });
    lvl.anchors.push(Anchor {
        time: 2000,
        fret: 7,
        width: 5,
        ..Default::default()
    });
    arr.levels.push(lvl);
    arr
}

/// Computes pi_times for the test arrangement (like create_phrase_iteration_times_array).
fn test_pi_times(arr: &InstrumentalArrangement) -> Vec<i32> {
    let mut times: Vec<i32> = arr.phrase_iterations.iter().map(|pi| pi.time).collect();
    times.push(arr.meta.song_length);
    times
}

/// Collects sorted note times from a level.
fn note_times_from_level(lvl: &Level) -> Vec<i32> {
    let mut times: Vec<i32> = lvl
        .notes
        .iter()
        .map(|n| n.time)
        .chain(lvl.chords.iter().map(|c| c.time))
        .collect();
    times.sort();
    times
}

fn ms_to_sec(ms: i32) -> f32 {
    ms as f32 / 1000.0
}

#[test]
fn beat_strong() {
    let b = Ebeat {
        time: 3666,
        measure: 2,
    };
    let test_arr = create_test_arr();
    let mut convert = make_beat_converter(&test_arr);

    let sng = convert(&b);

    assert!((sng.time - ms_to_sec(b.time)).abs() < 1e-3, "Time is same");
    assert_eq!(sng.measure, b.measure, "Measure is correct");
    assert_eq!(sng.beat, 0, "Beat is correct");
    assert!(
        sng.mask
            .contains(rocksmith2014_sng::BeatMask::FIRST_BEAT_OF_MEASURE),
        "First beat flag is set"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::BeatMask::EVEN_MEASURE),
        "Even measure flag is set"
    );
}

#[test]
fn beat_weak() {
    let b = Ebeat {
        time: 3666,
        measure: -1,
    };
    let test_arr = create_test_arr();
    let mut convert = make_beat_converter(&test_arr);

    let sng = convert(&b);

    assert!(
        !sng.mask
            .contains(rocksmith2014_sng::BeatMask::FIRST_BEAT_OF_MEASURE),
        "First beat flag is not set"
    );
}

#[test]
fn beats() {
    let beats = [
        Ebeat {
            time: 3000,
            measure: 1,
        },
        Ebeat {
            time: 3100,
            measure: -1,
        },
        Ebeat {
            time: 3200,
            measure: -1,
        },
        Ebeat {
            time: 3300,
            measure: 2,
        },
        Ebeat {
            time: 3400,
            measure: -1,
        },
    ];
    let test_arr = create_test_arr();
    let convert = make_beat_converter(&test_arr);

    let sng: Vec<_> = beats.iter().map(convert).collect();

    assert!(
        sng[0]
            .mask
            .contains(rocksmith2014_sng::BeatMask::FIRST_BEAT_OF_MEASURE),
        "B0: First beat flag is set"
    );
    assert!(
        !sng[0]
            .mask
            .contains(rocksmith2014_sng::BeatMask::EVEN_MEASURE),
        "B0: Even measure flag is not set"
    );
    assert_eq!(sng[1].beat, 1, "B1: Is second beat of measure");
    assert_eq!(sng[2].measure, 1, "B2: Is in measure 1");
    assert_eq!(sng[3].measure, 2, "B3: Is in measure 2");
    assert!(
        sng[3]
            .mask
            .contains(rocksmith2014_sng::BeatMask::EVEN_MEASURE),
        "B3: Even measure flag is set"
    );
    assert_eq!(sng[4].measure, 2, "B4: Is in measure 2");
    assert_eq!(sng[4].beat, 1, "B4: Is second beat of measure");
}

#[test]
fn vocal() {
    let v = rocksmith2014_xml::Vocal {
        time: 54_132,
        length: 22_222,
        lyric: "Hello".into(),
        note: 77,
    };

    let sng = xml_convert_vocal(&v);
    assert!((sng.time - ms_to_sec(v.time)).abs() < 1e-3, "Time is same");
    assert!(
        (sng.length - ms_to_sec(v.length)).abs() < 1e-3,
        "Length is same"
    );
    assert_eq!(&sng.lyric[..5], b"Hello", "Lyric is same");
    assert_eq!(sng.note, v.note as i32, "Note is same");
}

#[test]
fn phrase() {
    let ph = XmlPhrase {
        name: "ttt".into(),
        max_difficulty: 15,
        disparity: 1,
        ignore: 1,
        solo: 1,
    };
    let test_arr = create_test_arr();

    let sng = xml_convert_phrase(&test_arr, 1, &ph);

    assert_eq!(sng.name[..3], [b't', b't', b't'], "Name is same");
    assert_eq!(
        sng.max_difficulty, ph.max_difficulty as i32,
        "Max difficulty is same"
    );
    assert_eq!(sng.solo, 1, "Solo is set correctly");
    assert_eq!(sng.disparity, 1, "Disparity is set correctly");
    assert_eq!(sng.ignore, 1, "Ignore is set correctly");
    assert_eq!(
        sng.iteration_count, 2,
        "Phrase iteration count is set correctly"
    );
}

#[test]
fn chord_template_conversion() {
    let mut ct = ChordTemplate {
        name: "EEE".into(),
        display_name: "EEE".into(),
        ..Default::default()
    };
    ct.fingers = [1, 2, 3, 4, 5, 6];
    ct.frets = [1, 2, 3, 4, 5, 6];
    let test_arr = create_test_arr();

    let sng = xml_convert_chord_template(&test_arr, &ct);

    assert_eq!(&sng.name[..3], b"EEE", "Name is same");
    assert_eq!(sng.fingers, ct.fingers, "Fingers are same");
    assert_eq!(sng.frets, ct.frets, "Frets are same");
}

#[test]
fn chord_template_midi_notes() {
    let ct = ChordTemplate {
        frets: [1, 2, 3, 4, 5, 6],
        ..Default::default()
    };
    let test_arr = create_test_arr();

    let sng = xml_convert_chord_template(&test_arr, &ct);

    // MIDI notes: standard [40,45,50,55,59,64] + tuning [-1,2,4,-5,3,-2] + fret
    assert_eq!(
        sng.notes[0], 40,
        "MIDI note 1 is correct (40 + 1 fret - 1 tuning)"
    );
    assert_eq!(
        sng.notes[1], 49,
        "MIDI note 2 is correct (45 + 2 fret + 2 tuning)"
    );
    assert_eq!(
        sng.notes[2], 57,
        "MIDI note 3 is correct (50 + 3 fret + 4 tuning)"
    );
    assert_eq!(
        sng.notes[3], 54,
        "MIDI note 4 is correct (55 + 4 fret - 5 tuning)"
    );
    assert_eq!(
        sng.notes[4], 67,
        "MIDI note 5 is correct (59 + 5 fret + 3 tuning)"
    );
    assert_eq!(
        sng.notes[5], 68,
        "MIDI note 6 is correct (64 + 6 fret - 2 tuning)"
    );
}

#[test]
fn chord_template_arpeggio() {
    let ct = ChordTemplate {
        name: "E".into(),
        display_name: "E-arp".into(),
        ..Default::default()
    };
    let test_arr = create_test_arr();

    let sng = xml_convert_chord_template(&test_arr, &ct);

    assert!(
        sng.mask.contains(rocksmith2014_sng::ChordMask::ARPEGGIO),
        "Arpeggio is set"
    );
}

#[test]
fn chord_template_nop() {
    let ct = ChordTemplate {
        name: "E".into(),
        display_name: "E-nop".into(),
        ..Default::default()
    };
    let test_arr = create_test_arr();

    let sng = xml_convert_chord_template(&test_arr, &ct);

    assert!(
        sng.mask.contains(rocksmith2014_sng::ChordMask::NOP),
        "Nop is set"
    );
}

#[test]
fn bend_value_conversion() {
    let bv = BendValue {
        time: 456465,
        step: 99.0,
        unk5: 0,
    };

    let sng = xml_convert_bend_value(&bv);

    assert!((sng.time - ms_to_sec(bv.time)).abs() < 1e-1, "Time is same");
    assert_eq!(sng.step, bv.step as f32, "Step is same");
}

#[test]
fn phrase_iteration_conversion() {
    let test_arr = create_test_arr();
    let pi = &test_arr.phrase_iterations[1];
    let pi_times = test_pi_times(&test_arr);

    let sng = xml_convert_phrase_iteration(&pi_times, 1, pi);

    assert_eq!(sng.phrase_id, pi.phrase_id as i32, "Phrase ID is same");
    assert!(
        (sng.start_time - ms_to_sec(pi.time)).abs() < 1e-3,
        "Start time is same"
    );
}

#[test]
fn section_conversion() {
    let test_arr = create_test_arr();
    let section = &test_arr.sections[0];
    let string_masks: Vec<Vec<i8>> = vec![vec![]; test_arr.sections.len()];

    let sng = xml_convert_section(&string_masks, &test_arr, 0, section);

    assert_eq!(&sng.name[..1], b"1", "Section name is same");
    assert!(
        (sng.start_time - ms_to_sec(section.start_time)).abs() < 1e-3,
        "Start time is same"
    );
}

#[test]
fn events_to_dnas() {
    let test_arr = create_test_arr();
    let dnas = xml_create_dnas(&test_arr);
    assert_eq!(dnas.len(), 4, "DNA count is correct");
    assert_eq!(dnas[3].dna_id, 2, "Last DNA ID is correct");
}

#[test]
fn meta_data() {
    let test_arr = create_test_arr();
    let accu = AccuData::init(&test_arr);
    let md = xml_create_meta_data(&accu, 10.0, &test_arr);
    assert_eq!(md.max_score, 100_000.0, "Max score is correct");
    assert_eq!(md.start_time, 1.0, "Start time is correct");
    assert_eq!(md.capo_fret_id, -1, "Capo fret is correct");
    assert_eq!(md.part, test_arr.meta.part as i16, "Part is same");
    assert!(
        (md.song_length - ms_to_sec(test_arr.meta.song_length)).abs() < 1e-3,
        "Song length is same"
    );
    assert_eq!(
        md.tuning,
        test_arr.meta.tuning.strings.to_vec(),
        "Tuning is same"
    );
}

#[test]
fn note_mask_1() {
    let note = Note {
        fret: 2,
        string: 3,
        time: 1000,
        sustain: 500,
        ..Note::default()
    };

    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(note.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &[],
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng = converter.call(0, XmlEntity::Note(note));

    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::SUSTAIN),
        "Sustained note has sustain flag"
    );
}

#[test]
fn note_mask_2() {
    let note = Note {
        fret: 2,
        string: 3,
        time: 1000,
        slide_to: 5,
        slide_unpitch_to: 5,
        tap: 1,
        vibrato: 40,
        left_hand: 1,
        bend_values: vec![BendValue {
            time: 1000,
            step: 1.0,
            unk5: 0,
        }],
        ..Note::default()
    };

    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(note.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &[],
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng = converter.call(0, XmlEntity::Note(note));

    assert!(
        !sng.mask.contains(rocksmith2014_sng::NoteMask::OPEN),
        "Non-open note does not have open flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::SLIDE),
        "Slide note has slide flag"
    );
    assert!(
        sng.mask
            .contains(rocksmith2014_sng::NoteMask::UNPITCHED_SLIDE),
        "Unpitched slide note has unpitched slide flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::TAP),
        "Tapped note has tap flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::VIBRATO),
        "Vibrato note has vibrato flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::BEND),
        "Bend note has bend flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::LEFT_HAND),
        "Note with left hand has left hand flag"
    );
}

#[test]
fn chord_conversion() {
    let chord = Chord {
        time: 1250,
        chord_id: 0,
        chord_notes: vec![
            rocksmith2014_xml::ChordNote {
                sustain: 500,
                ..Default::default()
            },
            rocksmith2014_xml::ChordNote {
                sustain: 500,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut test_arr = create_test_arr();
    test_arr.levels[0].chords.push(chord.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &[],
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng = converter.call(0, XmlEntity::Chord(chord));

    assert_eq!(sng.chord_id, 0, "Chord ID is same");
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::CHORD),
        "Chord has chord flag"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::SUSTAIN),
        "Sustain flag is set"
    );
    assert!(
        sng.mask.contains(rocksmith2014_sng::NoteMask::CHORD_NOTES),
        "Chord notes flag is set"
    );
}

#[test]
fn level_conversion() {
    let mut test_arr = create_test_arr();
    let pi_times = test_pi_times(&test_arr);
    test_arr.phrases.push(XmlPhrase {
        name: "default".into(),
        ..Default::default()
    });
    test_arr.phrases.push(XmlPhrase {
        name: "phrase1".into(),
        ..Default::default()
    });
    let test_level = test_arr.levels[0].clone();
    let mut accu = AccuData::init(&test_arr);

    let sng = xml_convert_level(&mut accu, &pi_times, &test_arr, &test_level);

    assert_eq!(
        sng.difficulty, test_level.difficulty as i32,
        "Difficulty is same"
    );
    assert_eq!(
        sng.anchors.len(),
        test_level.anchors.len(),
        "Anchor count is same"
    );
}

#[test]
fn phrase_iteration_last() {
    let test_arr = create_test_arr();
    let pi = &test_arr.phrase_iterations[4];
    let pi_times = test_pi_times(&test_arr);

    let sng = xml_convert_phrase_iteration(&pi_times, 4, pi);

    assert_eq!(sng.phrase_id, pi.phrase_id as i32, "Phrase ID is the same");
    assert!(
        (sng.start_time - ms_to_sec(pi.time)).abs() < 1e-3,
        "Start time is the same"
    );
    assert!(
        (sng.end_time - ms_to_sec(test_arr.meta.song_length)).abs() < 1e-3,
        "End time uses song length for the last phrase iteration"
    );
}

#[test]
fn new_linked_difficulty() {
    let nld = NewLinkedDiff {
        level_break: 2,
        phrase_ids: vec![3, 5, 7],
    };

    let sng = xml_convert_new_linked_difficulty(&nld);

    assert_eq!(
        sng.level_break, nld.level_break as i32,
        "Level break is the same"
    );
    assert_eq!(sng.nld_phrases, nld.phrase_ids, "Phrase IDs are the same");
}

#[test]
fn event_conversion() {
    let e = ArrangementEvent {
        time: 1_750_735,
        code: "wedge_cutoff".into(),
    };

    let sng = xml_convert_event(&e);
    assert!((sng.time - ms_to_sec(e.time)).abs() < 1e-3, "Time is same");
    assert_eq!(&sng.name[..12], b"wedge_cutoff", "Code/name is same");
}

#[test]
fn tone_conversion() {
    let tone = ToneChange {
        time: 3_215_123,
        name: "tone_test".into(),
        id: 3,
    };

    let sng = xml_convert_tone(&tone);
    assert!(
        (sng.time - ms_to_sec(tone.time)).abs() < 1e-3,
        "Time is the same"
    );
    assert_eq!(sng.tone_id, tone.id, "Tone ID is same");
}

#[test]
fn section_last() {
    let test_arr = create_test_arr();
    let section = &test_arr.sections[2];
    let string_masks: Vec<Vec<i8>> = vec![vec![]; test_arr.sections.len()];

    let sng = xml_convert_section(&string_masks, &test_arr, 2, section);

    assert!(
        (sng.start_time - ms_to_sec(section.start_time)).abs() < 1e-3,
        "Start time is same"
    );
    assert!(
        (sng.end_time - ms_to_sec(test_arr.meta.song_length)).abs() < 1e-3,
        "End time uses song length for the last section"
    );
    assert_eq!(
        sng.start_phrase_iteration_id, 4,
        "Start PI index is correct"
    );
    assert_eq!(sng.end_phrase_iteration_id, 4, "End PI index is correct");
}

#[test]
fn section_phrase_iteration_start_end_1_iteration() {
    let mut test_arr = create_test_arr();
    test_arr.phrase_iterations.truncate(1);
    let section = &test_arr.sections[0];
    let string_masks: Vec<Vec<i8>> = vec![vec![]; test_arr.sections.len()];

    let sng = xml_convert_section(&string_masks, &test_arr, 0, section);

    assert_eq!(
        sng.start_phrase_iteration_id, 0,
        "Start PI index is correct"
    );
    assert_eq!(sng.end_phrase_iteration_id, 0, "End PI index is correct");
}

#[test]
fn section_phrase_iteration_start_end_3_phrase_iterations() {
    let test_arr = create_test_arr();
    let section = &test_arr.sections[0];
    let string_masks: Vec<Vec<i8>> = vec![vec![]; test_arr.sections.len()];

    let sng = xml_convert_section(&string_masks, &test_arr, 0, section);

    assert_eq!(
        sng.start_phrase_iteration_id, 0,
        "Start PI index is correct"
    );
    assert_eq!(sng.end_phrase_iteration_id, 2, "End PI index is correct");
}

#[test]
fn anchor_conversion() {
    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(Note {
        time: 1_500,
        sustain: 100,
        ..Default::default()
    });
    test_arr.levels[0].notes.push(Note {
        time: 1_900,
        sustain: 50,
        ..Default::default()
    });
    let level = &test_arr.levels[0];
    let note_times = note_times_from_level(level);
    let xml_notes = level.notes.clone();
    let anchor = &level.anchors[0];

    let sng = xml_convert_anchor(&xml_notes, &note_times, level, &test_arr, 0, anchor);

    assert!(
        (sng.start_time - ms_to_sec(anchor.time)).abs() < 1e-3,
        "Start time is same"
    );
    assert!(
        (sng.end_time - 2.0).abs() < 1e-3,
        "End time is next anchor time"
    );
    assert!(
        (sng.first_note_time - 1.5).abs() < 1e-3,
        "First note time is same"
    );
    assert!(
        (sng.last_note_time - 1.9).abs() < 1e-3,
        "Last note time is same"
    );
    assert_eq!(sng.fret_id, anchor.fret, "Fret is same");
    assert_eq!(sng.width, anchor.width, "Width is same");
    assert_eq!(sng.phrase_iteration_id, 0, "Phrase iteration id is same");
}

#[test]
fn hand_shape_conversion() {
    let hs = HandShape {
        chord_id: 3,
        start_time: 1_400,
        end_time: 1_800,
    };
    let n1 = Note {
        time: 1_500,
        sustain: 100,
        ..Default::default()
    };
    let n2 = Note {
        time: 1_700,
        sustain: 50,
        ..Default::default()
    };
    let note_times = vec![n1.time, n2.time];
    let entities = vec![XmlEntity::Note(n1), XmlEntity::Note(n2)];

    let fp = xml_convert_handshape(&note_times, &entities, &hs);

    assert_eq!(fp.chord_id, hs.chord_id, "Chord ID is same");
    assert!((fp.start_time - 1.4).abs() < 1e-3, "Start time is same");
    assert!((fp.end_time - 1.8).abs() < 1e-3, "End time is same");
    assert!(
        (fp.first_note_time - 1.5).abs() < 1e-3,
        "First note time is same"
    );
    assert!(
        (fp.last_note_time - 1.7).abs() < 1e-3,
        "Last note time is same"
    );
}

#[test]
fn hand_shape_last_note_time_for_sustained_chord() {
    let hs = HandShape {
        chord_id: 2,
        start_time: 1_400,
        end_time: 1_800,
    };
    let note = Note {
        time: 1_500,
        sustain: 500,
        ..Default::default()
    };
    let note_times = vec![note.time];
    let entities = vec![XmlEntity::Note(note)];

    let fp = xml_convert_handshape(&note_times, &entities, &hs);

    assert_eq!(fp.chord_id, hs.chord_id, "Chord ID is same");
    assert!(
        (fp.first_note_time - 1.5).abs() < 1e-3,
        "First note time is same"
    );
    assert_eq!(
        fp.last_note_time, -1.0,
        "Last note time is -1.0 when sustain extends to hand shape end"
    );
}

#[test]
#[ignore = "Parity placeholder: Note conversion coverage not implemented yet"]
fn note_conversion() {}

#[test]
fn note_next_previous_note_ids() {
    let n1 = Note {
        time: 1000,
        string: 1,
        fret: 3,
        ..Default::default()
    };
    let n2 = Note {
        time: 1500,
        string: 2,
        fret: 5,
        ..Default::default()
    };

    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(n1.clone());
    test_arr.levels[0].notes.push(n2.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &[],
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng1 = converter.call(0, XmlEntity::Note(n1));
    let sng2 = converter.call(1, XmlEntity::Note(n2));

    assert_eq!(sng1.prev_iter_note, -1, "First note has no previous note");
    assert_eq!(sng1.next_iter_note, 1, "First note points to next note");
    assert_eq!(
        sng2.prev_iter_note, 0,
        "Second note points to previous note"
    );
    assert_eq!(sng2.next_iter_note, -1, "Second note has no next note");
}

#[test]
fn note_link_next() {
    let n1 = Note {
        time: 1000,
        string: 1,
        fret: 3,
        mask: rocksmith2014_xml::NoteMask::LINK_NEXT,
        ..Default::default()
    };
    let n2 = Note {
        time: 1500,
        string: 1,
        fret: 5,
        ..Default::default()
    };

    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(n1.clone());
    test_arr.levels[0].notes.push(n2.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &[],
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng1 = converter.call(0, XmlEntity::Note(n1));
    let sng2 = converter.call(1, XmlEntity::Note(n2));

    assert!(
        sng1.mask.contains(rocksmith2014_sng::NoteMask::PARENT),
        "Link-next note has parent flag"
    );
    assert!(
        sng2.mask.contains(rocksmith2014_sng::NoteMask::CHILD),
        "Following note on same string has child flag"
    );
    assert_eq!(
        sng2.parent_prev_note, 0,
        "Child note points to parent link-next index"
    );
}

#[test]
fn note_hand_shape_id() {
    let note = Note {
        time: 1500,
        string: 1,
        fret: 5,
        ..Default::default()
    };
    let hand_shapes = vec![rocksmith2014_sng::FingerPrint {
        chord_id: 3,
        start_time: 1.4,
        end_time: 1.8,
        first_note_time: 1.5,
        last_note_time: 1.5,
    }];

    let mut test_arr = create_test_arr();
    test_arr.levels[0].notes.push(note.clone());

    let note_times = note_times_from_level(&test_arr.levels[0]);
    let pi_times = test_pi_times(&test_arr);
    let mut accu = AccuData::init(&test_arr);
    let mut converter = NoteConverter::new(
        &note_times,
        &pi_times,
        &hand_shapes,
        &[],
        &mut accu,
        flag_on_anchor_change,
        &test_arr,
        0,
    );

    let sng = converter.call(0, XmlEntity::Note(note));

    assert_eq!(
        sng.finger_print_id[0], 0,
        "Hand shape ID references the first fingerprint (index 0)"
    );
    assert_eq!(sng.finger_print_id[1], -1, "Arpeggio ID remains unset");
}

#[test]
#[ignore = "Parity placeholder: Note hand shape id arpeggio behavior not implemented yet"]
fn note_hand_shape_id_arpeggio() {}

#[test]
#[ignore = "Parity placeholder: Chord double stop/arpeggio/no chord notes not implemented yet"]
fn chord_double_stop_arpeggio_no_chord_notes() {}

#[test]
#[ignore = "Parity placeholder: Chord mask behavior not implemented yet"]
fn chord_mask() {}

#[test]
#[ignore = "Parity placeholder: Chord link next behavior not implemented yet"]
fn chord_link_next() {}

#[test]
#[ignore = "Parity placeholder: Chord notes creation behavior not implemented yet"]
fn chord_notes_are_created_when_needed() {}

#[test]
#[ignore = "Parity placeholder: Chord notes omission behavior not implemented yet"]
fn chord_notes_are_not_created_when_not_needed() {}

#[test]
#[ignore = "Parity placeholder: Anchor extensions for slide notes not implemented yet"]
fn anchor_extensions_are_created_for_slide_notes() {}

#[test]
fn section_string_mask() {
    let test_arr = create_test_arr();
    let section = &test_arr.sections[1];
    let mut string_masks: Vec<Vec<i8>> = vec![vec![]; test_arr.sections.len()];
    string_masks[1] = vec![1, 0, -1, 5];

    let sng = xml_convert_section(&string_masks, &test_arr, 1, section);

    assert_eq!(sng.string_mask[0], 1, "First mask value is copied");
    assert_eq!(sng.string_mask[1], 0, "Second mask value is copied");
    assert_eq!(sng.string_mask[2], -1, "Third mask value is copied");
    assert_eq!(sng.string_mask[3], 5, "Fourth mask value is copied");
    assert_eq!(sng.string_mask[4], 0, "Remaining values are default");
}
