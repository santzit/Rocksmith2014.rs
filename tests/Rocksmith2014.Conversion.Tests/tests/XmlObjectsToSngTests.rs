//! XML Objects → SNG Objects conversion tests.
//!
//! Mirrors `XmlObjectsToSngTests.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{
    flag_on_anchor_change, make_beat_converter, map_to_midi_notes, xml_convert_bend_value,
    xml_convert_chord_template, xml_convert_event, xml_convert_handshape, xml_convert_level,
    xml_convert_phrase, xml_convert_phrase_iteration, xml_convert_section, xml_convert_tone,
    AccuData, NoteConverter, XmlEntity,
};
use rocksmith2014_xml::{
    Anchor, ArrangementEvent, BendValue, ChordTemplate, Ebeat, HandShape, InstrumentalArrangement,
    Level, Note, PhraseIteration, Section, ToneChange,
};
use rocksmith2014_xml::{Chord, ChordMask, MetaData, Phrase as XmlPhrase};

fn create_test_arr() -> InstrumentalArrangement {
    let mut arr = InstrumentalArrangement::default();
    arr.meta.song_length = 4784_455;

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
        time: 7554_100,
        phrase_id: 2,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 7555_000,
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
        start_time: 8000_000,
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
    let mut convert = make_beat_converter(&test_arr);

    let sng: Vec<_> = beats.iter().map(|b| convert(b)).collect();

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
#[ignore = "convert_vocal (single) is not publicly exported from rocksmith2014-conversion"]
fn vocal() {}

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
    let mut ct = ChordTemplate::default();
    ct.frets = [1, 2, 3, 4, 5, 6];
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
#[ignore = "create_dnas is not publicly exported from rocksmith2014-conversion"]
fn events_to_dnas() {}

#[test]
#[ignore = "create_meta_data is not publicly exported from rocksmith2014-conversion"]
fn meta_data() {}

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
