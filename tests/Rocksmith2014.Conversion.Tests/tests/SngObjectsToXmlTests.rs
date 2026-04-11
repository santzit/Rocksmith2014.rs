//! SNG Objects → XML Objects conversion tests.
//!
//! Mirrors `SngObjectsToXmlTests.fs` in Rocksmith2014.Conversion.Tests (.NET).

use rocksmith2014_conversion::{
    sng_convert_anchor, sng_convert_beat, sng_convert_bend_data32, sng_convert_bend_value,
    sng_convert_chord, sng_convert_chord_template, sng_convert_event, sng_convert_hand_shape,
    sng_convert_level, sng_convert_new_linked_difficulty, sng_convert_note, sng_convert_phrase,
    sng_convert_phrase_extra_info, sng_convert_phrase_iteration, sng_convert_section,
    sng_convert_symbol_definition, sng_convert_tone, sng_convert_vocal,
};
use rocksmith2014_sng::{
    Anchor, Beat, BeatMask, BendData32, BendValue, Chord, ChordMask, ChordNotes, Event,
    FingerPrint, Level, NewLinkedDifficulty, Note, NoteMask, PhraseExtraInfo, PhraseIteration,
    Rect, Section, Sng, SymbolDefinition, Tone, Vocal,
};
use rocksmith2014_xml::NoteMask as XmlNoteMask;

fn make_name_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut buf = [0u8; N];
    for (i, b) in s.bytes().enumerate().take(N - 1) {
        buf[i] = b;
    }
    buf
}

fn test_sng() -> Sng {
    let chord = Chord {
        frets: [-1, 0, 2, 2, 2, 0],
        fingers: [-1, -1, 1, 2, 3, -1],
        name: make_name_bytes::<32>("A"),
        ..Default::default()
    };

    let mut sng = Sng::default();
    sng.chords.push(chord);
    sng.chord_notes.push(ChordNotes::default());
    sng
}

#[test]
fn beat() {
    let b = Beat {
        time: 5468.422,
        measure: 5,
        beat: 0,
        phrase_iteration: 0,
        mask: BeatMask::FIRST_BEAT_OF_MEASURE,
    };

    let xml = sng_convert_beat(&b);

    assert_eq!(xml.time, 5_468_422, "Time is same");
    assert_eq!(xml.measure, b.measure, "Measure is same");
}

#[test]
fn chord_template() {
    let c = Chord {
        name: make_name_bytes::<32>("Eb9/A#"),
        fingers: [-1, 4, 3, 2, 1, -1],
        frets: [-1, 5, 6, 7, 8, -1],
        ..Default::default()
    };

    let xml = sng_convert_chord_template(&c);

    assert_eq!(xml.display_name, "Eb9/A#", "Display name is correct");
    assert_eq!(xml.name, "Eb9/A#", "Chord name is same");
    assert_eq!(xml.fingers, c.fingers, "Fingering is same");
    assert_eq!(xml.frets, c.frets, "Frets are same");
}

#[test]
fn chord_template_arpeggio() {
    let c = Chord {
        name: make_name_bytes::<32>("Eb9/A#"),
        mask: ChordMask::ARPEGGIO,
        ..Default::default()
    };

    let xml = sng_convert_chord_template(&c);

    assert_eq!(xml.display_name, "Eb9/A#-arp", "Display name is correct");
}

#[test]
fn chord_template_nop() {
    let c = Chord {
        name: make_name_bytes::<32>("Eb9/A#"),
        mask: ChordMask::NOP,
        ..Default::default()
    };

    let xml = sng_convert_chord_template(&c);

    assert_eq!(xml.display_name, "Eb9/A#-nop", "Display name is correct");
}

#[test]
fn phrase() {
    use rocksmith2014_sng::Phrase;
    let p = Phrase {
        solo: 1,
        disparity: 1,
        ignore: 1,
        max_difficulty: 25,
        name: make_name_bytes::<32>("thelittleguitarthatcould"),
        ..Default::default()
    };

    let xml = sng_convert_phrase(&p);

    assert_eq!(xml.name, "thelittleguitarthatcould", "Name is same");
    assert_eq!(xml.max_difficulty, 25, "Max difficulty is same");
    assert_eq!(xml.solo, 1, "Is solo phrase");
    assert_eq!(xml.disparity, 1, "Is disparity phrase");
    assert_eq!(xml.ignore, 1, "Is ignore phrase");
}

#[test]
fn bend_value() {
    let bv = BendValue {
        time: 11.111,
        step: 2.5,
        unused: 0,
    };

    let xml = sng_convert_bend_value(&bv);

    assert_eq!(xml.step, bv.step as f64, "Step is same");
    assert_eq!(xml.time, 11_111, "Time code is same");
}

#[test]
fn bend_data32_empty() {
    let bd = BendData32::default();
    let xml = sng_convert_bend_data32(&bd);
    assert!(xml.is_none(), "None is returned for empty bend data");
}

#[test]
fn bend_data32() {
    let mut bd = BendData32::default();
    bd.bend_values[0] = BendValue {
        time: 11.111,
        step: 2.5,
        unused: 0,
    };
    bd.bend_values[1] = BendValue {
        time: 22.222,
        step: 1.5,
        unused: 0,
    };
    bd.used_count = 2;

    let xml = sng_convert_bend_data32(&bd).expect("bend data should be present");
    assert_eq!(xml.len(), bd.used_count as usize, "Count is same");
    assert_eq!(xml[0].time, 11_111, "Time code of first bend value is same");
    assert_eq!(xml[1].step, 1.5, "Step of second bend value is same");
}

#[test]
fn vocal() {
    let mut lyric = [0u8; 48];
    lyric[..4].copy_from_slice(b"end+");
    let v = Vocal {
        time: 87.999,
        note: 77,
        length: 4.654,
        lyric,
    };

    let xml = sng_convert_vocal(&v);
    assert_eq!(xml.lyric, "end+", "Lyric is same");
    assert_eq!(xml.time, 87_999, "Time code is same");
    assert_eq!(xml.length, 4_654, "Length is same");
    assert_eq!(xml.note, 77, "Note is same");
}

#[test]
fn symbol_definition() {
    let mut symbol = [0u8; 12];
    let bytes = "轟".as_bytes();
    symbol[..bytes.len()].copy_from_slice(bytes);
    let sd = SymbolDefinition {
        symbol,
        outer: Rect {
            ymin: 0.12,
            ymax: 0.77,
            xmin: 0.05,
            xmax: 1.7,
        },
        inner: Rect {
            ymin: 4.7,
            ymax: 1.11,
            xmin: 55.5,
            xmax: 2.8,
        },
    };

    let xml = sng_convert_symbol_definition(&sd);
    assert_eq!(xml.symbol, "轟", "Symbol is same");
    assert_eq!(xml.outer_y_min, sd.outer.ymin, "Outer Y Min is same");
    assert_eq!(xml.outer_y_max, sd.outer.ymax, "Outer Y Max is same");
    assert_eq!(xml.outer_x_min, sd.outer.xmin, "Outer X Min is same");
    assert_eq!(xml.outer_x_max, sd.outer.xmax, "Outer X Max is same");
    assert_eq!(xml.inner_y_min, sd.inner.ymin, "Inner Y Min is same");
    assert_eq!(xml.inner_y_max, sd.inner.ymax, "Inner Y Max is same");
    assert_eq!(xml.inner_x_min, sd.inner.xmin, "Inner X Min is same");
    assert_eq!(xml.inner_x_max, sd.inner.xmax, "Inner X Max is same");
}

#[test]
fn phrase_iteration() {
    let pi = PhraseIteration {
        phrase_id: 44,
        start_time: 44.217,
        end_time: 45.001,
        difficulty: [5, 8, 13],
    };

    let xml = sng_convert_phrase_iteration(&pi);

    assert_eq!(xml.phrase_id, pi.phrase_id as u32, "Phrase ID is same");
    assert_eq!(xml.time, 44_217, "Time code is same");
    let hls = xml.hero_levels.as_deref().unwrap_or(&[]);
    assert_eq!(hls[0].difficulty, pi.difficulty[0], "Easy is same level");
    assert_eq!(hls[1].difficulty, pi.difficulty[1], "Medium is same level");
    assert_eq!(hls[2].difficulty, pi.difficulty[2], "Hard is same level");
}

#[test]
fn phrase_properties() {
    let pi = PhraseExtraInfo {
        phrase_id: 5,
        difficulty: 3,
        empty: 7,
        level_jump: 1,
        redundant: 12,
    };

    let xml = sng_convert_phrase_extra_info(&pi);

    assert_eq!(xml.phrase_id, pi.phrase_id, "Phrase ID is same");
    assert_eq!(xml.difficulty, pi.difficulty, "Difficulty is same");
    assert_eq!(xml.empty, pi.empty, "Empty is same");
    assert_eq!(xml.level_jump, pi.level_jump as i32, "Level jump is same");
    assert_eq!(xml.redundant, pi.redundant as i32, "Redundant is same");
}

#[test]
fn new_linked_difficulty() {
    let nld = NewLinkedDifficulty {
        level_break: 12,
        nld_phrases: vec![2, 6, 15],
    };

    let xml = sng_convert_new_linked_difficulty(&nld);
    assert_eq!(
        xml.level_break, nld.level_break as i8,
        "Level break is same"
    );
    assert_eq!(xml.phrase_ids, nld.nld_phrases, "Phrase IDs are same");
}

#[test]
fn event() {
    let e = Event {
        time: 1750.735,
        name: make_name_bytes::<256>("wedge_cutoff"),
    };

    let xml = sng_convert_event(&e);

    assert_eq!(xml.code, "wedge_cutoff", "Code/name is same");
    assert_eq!(xml.time, 1_750_735, "Time code is same");
}

#[test]
fn tone() {
    let t = Tone {
        time: 4568.0,
        tone_id: 2,
    };
    // ToneId=2 means Tone_C; pass tone names indexed by id
    let tone_names = &["", "", "tone_c"];

    let xml = sng_convert_tone(&t, tone_names);

    assert_eq!(xml.id, t.tone_id, "Tone ID is same");
    assert_eq!(xml.time, 4_568_000, "Time code is same");
    assert_eq!(xml.name, "tone_c", "Tone name is correct");
}

#[test]
fn section() {
    let s = Section {
        name: make_name_bytes::<32>("chorus"),
        number: 3,
        start_time: 123.456,
        end_time: 789.012,
        ..Default::default()
    };

    let xml = sng_convert_section(&s);

    assert_eq!(xml.name, "chorus", "Section name is same");
    assert_eq!(xml.start_time, 123_456, "Time code is same");
    assert_eq!(xml.number, 3, "Section number is same");
}

#[test]
fn anchor() {
    let a = Anchor {
        start_time: 5.0,
        end_time: 6.0,
        first_note_time: 5.0,
        last_note_time: f32::NAN,
        fret_id: 14,
        width: 4,
        phrase_iteration_id: 3,
    };

    let xml = sng_convert_anchor(&a);

    assert_eq!(xml.fret, a.fret_id, "Fret is same");
    assert_eq!(xml.time, 5_000, "Time code is same");
    assert_eq!(xml.width, a.width, "Width is same");
}

#[test]
fn finger_print_hand_shape() {
    let fp = FingerPrint {
        chord_id: 15,
        start_time: 999.999,
        end_time: 1001.001,
        first_note_time: 999.999,
        last_note_time: 1001.0,
    };

    let xml = sng_convert_hand_shape(&fp);

    assert_eq!(xml.chord_id, fp.chord_id, "Chord ID is same");
    assert_eq!(xml.start_time, 999_999, "Start time is same");
    assert_eq!(xml.end_time, 1_001_001, "End time is same");
}

#[test]
fn note() {
    let n = Note {
        mask: NoteMask::SINGLE
            | NoteMask::HAMMER_ON
            | NoteMask::ACCENT
            | NoteMask::MUTE
            | NoteMask::HARMONIC
            | NoteMask::IGNORE
            | NoteMask::PARENT
            | NoteMask::PALM_MUTE
            | NoteMask::PINCH_HARMONIC
            | NoteMask::TREMOLO
            | NoteMask::RIGHT_HAND
            | NoteMask::PULL_OFF
            | NoteMask::SLAP
            | NoteMask::PLUCK,
        flags: 0,
        hash: 1234,
        time: 55.55,
        string_index: 4,
        fret: 8,
        anchor_fret: 8,
        anchor_width: 4,
        chord_id: -1,
        chord_notes_id: -1,
        phrase_id: 7,
        phrase_iteration_id: 12,
        finger_print_id: [-1, -1],
        next_iter_note: 16,
        prev_iter_note: 14,
        parent_prev_note: 14,
        slide_to: 10,
        slide_unpitch_to: 12,
        left_hand: 2,
        tap: 3,
        pick_direction: -1,
        slap: 1,
        pluck: 1,
        vibrato: 120,
        sustain: 15.0,
        max_bend: 1.0,
        bend_data: vec![BendValue {
            time: 55.661,
            step: 1.0,
            unused: 0,
        }],
    };

    let xml = sng_convert_note(&n);

    assert_eq!(xml.time, 55_550, "Time code is same");
    assert_eq!(xml.string, n.string_index, "String is same");
    assert_eq!(xml.fret, n.fret, "Fret is same");
    assert_eq!(xml.sustain, 15_000, "Sustain is same");
    assert_eq!(xml.vibrato, n.vibrato as i8, "Vibrato is same");
    assert_eq!(xml.slide_to, n.slide_to, "Slide is same");
    assert_eq!(
        xml.slide_unpitch_to, n.slide_unpitch_to,
        "Unpitched slide is same"
    );
    assert_eq!(xml.left_hand, n.left_hand, "Left hand is same");
    assert_eq!(xml.tap, n.tap, "Tap is same");
    assert!(xml.slap >= 0, "Slap is same");
    assert!(xml.pluck >= 0, "Pluck is same");
    assert!(
        xml.mask.contains(XmlNoteMask::HAMMER_ON),
        "Hammer-on is same"
    );
    assert!(xml.mask.contains(XmlNoteMask::PULL_OFF), "Pull-off is same");
    assert!(xml.mask.contains(XmlNoteMask::ACCENT), "Accent is same");
    assert!(
        xml.mask.contains(XmlNoteMask::FRET_HAND_MUTE),
        "Fret-hand mute is same"
    );
    assert!(xml.mask.contains(XmlNoteMask::HARMONIC), "Harmonic is same");
    assert!(xml.mask.contains(XmlNoteMask::IGNORE), "Ignore is same");
    assert!(
        xml.mask.contains(XmlNoteMask::LINK_NEXT),
        "Link-next is same"
    );
    assert!(
        xml.mask.contains(XmlNoteMask::PALM_MUTE),
        "Palm-mute is same"
    );
    assert!(
        xml.mask.contains(XmlNoteMask::PINCH_HARMONIC),
        "Pinch harmonic is same"
    );
    assert!(xml.mask.contains(XmlNoteMask::TREMOLO), "Tremolo is same");
    assert!(
        xml.mask.contains(XmlNoteMask::RIGHT_HAND),
        "Right hand is same"
    );
    assert_eq!(xml.max_bend, n.max_bend as f64, "Max bend is same");
    assert_eq!(
        xml.bend_values.len(),
        n.bend_data.len(),
        "Bend value count is same"
    );
}

#[test]
fn chord_note() {
    let sng = test_sng();
    let c = Note {
        mask: NoteMask::CHORD
            | NoteMask::PARENT
            | NoteMask::ACCENT
            | NoteMask::FRET_HAND_MUTE
            | NoteMask::HIGH_DENSITY
            | NoteMask::IGNORE
            | NoteMask::PALM_MUTE
            | NoteMask::CHORD_PANEL,
        flags: 0,
        hash: 1234,
        time: 66.66,
        string_index: -1,
        fret: -1,
        anchor_fret: 8,
        anchor_width: 4,
        chord_id: 0,
        chord_notes_id: 0,
        phrase_id: 7,
        phrase_iteration_id: 12,
        finger_print_id: [1, -1],
        next_iter_note: 16,
        prev_iter_note: 14,
        parent_prev_note: 14,
        slide_to: -1,
        slide_unpitch_to: -1,
        left_hand: -1,
        tap: -1,
        pick_direction: -1,
        slap: -1,
        pluck: -1,
        vibrato: 0,
        sustain: 0.0,
        max_bend: 0.0,
        bend_data: vec![],
    };

    let xml = sng_convert_chord(&sng, &c);

    assert_eq!(xml.time, 66_660, "Time code is same");
    assert_eq!(xml.chord_id, c.chord_id, "Chord ID is same");
    assert!(
        xml.mask.contains(rocksmith2014_xml::ChordMask::LINK_NEXT),
        "Link-next is same"
    );
    assert!(
        xml.mask.contains(rocksmith2014_xml::ChordMask::ACCENT),
        "Accent is same"
    );
    assert!(
        xml.mask
            .contains(rocksmith2014_xml::ChordMask::FRET_HAND_MUTE),
        "Fret-hand mute is same"
    );
    assert!(
        xml.mask
            .contains(rocksmith2014_xml::ChordMask::HIGH_DENSITY),
        "High density is same"
    );
    assert!(
        xml.mask.contains(rocksmith2014_xml::ChordMask::IGNORE),
        "Ignore is same"
    );
    assert!(
        xml.mask.contains(rocksmith2014_xml::ChordMask::PALM_MUTE),
        "Palm-mute is same"
    );
    // Template has frets [-1,0,2,2,2,0] → 5 non-(-1) entries
    assert_eq!(xml.chord_notes.len(), 5, "Chord notes were created");
}

#[test]
fn chord_no_chord_notes() {
    let sng = test_sng();
    let c = Note {
        mask: NoteMask::CHORD,
        chord_id: 0,
        chord_notes_id: -1,
        time: 66.66,
        ..Note::default()
    };

    let xml = sng_convert_chord(&sng, &c);

    assert!(xml.chord_notes.is_empty(), "Chord notes were not created");
}

#[test]
fn level() {
    let sng = test_sng();

    let a = Anchor {
        start_time: 10.0,
        end_time: 11.0,
        first_note_time: 10.0,
        last_note_time: 11.0,
        fret_id: 4,
        width: 7,
        phrase_iteration_id: 1,
    };

    let fp1 = FingerPrint {
        chord_id: 0,
        start_time: 10.5,
        end_time: 10.75,
        first_note_time: 10.5,
        last_note_time: 10.5,
    };

    let fp2 = FingerPrint {
        chord_id: 1,
        start_time: 10.82,
        end_time: 10.99,
        first_note_time: 10.82,
        last_note_time: 10.90,
    };

    let n = Note {
        mask: NoteMask::SINGLE | NoteMask::HAMMER_ON,
        time: 55.55,
        string_index: 4,
        fret: 8,
        chord_id: -1,
        chord_notes_id: -1,
        slide_to: 10,
        slide_unpitch_to: 12,
        left_hand: 2,
        tap: 3,
        slap: 1,
        pluck: 1,
        vibrato: 120,
        sustain: 15.0,
        max_bend: 1.0,
        ..Note::default()
    };

    let c = Note {
        mask: NoteMask::CHORD,
        time: 66.66,
        chord_id: 0,
        chord_notes_id: 0,
        finger_print_id: [1, -1],
        ..Note::default()
    };

    let lvl = Level {
        difficulty: 4,
        anchors: vec![a],
        anchor_extensions: vec![],
        hand_shapes: vec![fp1],
        arpeggios: vec![fp2],
        notes: vec![n, c],
        average_notes_per_iteration: vec![1.0],
        notes_in_phrase_iterations_excl_ignored: vec![1],
        notes_in_phrase_iterations_all: vec![1],
    };

    let xml = sng_convert_level(&sng, &lvl);

    assert_eq!(xml.difficulty, lvl.difficulty as i8, "Difficulty is same");
    assert_eq!(xml.anchors.len(), lvl.anchors.len(), "Anchor count is same");
    assert_eq!(
        xml.hand_shapes.len(),
        lvl.hand_shapes.len() + lvl.arpeggios.len(),
        "Handshape count is same"
    );
    assert_eq!(
        xml.notes.len() + xml.chords.len(),
        lvl.notes.len(),
        "Note/chord count is same"
    );
}
