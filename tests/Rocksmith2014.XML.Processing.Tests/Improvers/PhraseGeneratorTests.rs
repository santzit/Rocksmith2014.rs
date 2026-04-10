use rocksmith2014_xml::{Anchor, Ebeat, InstrumentalArrangement, Level, Note};
use rocksmith2014_xml_processing::phrase_gen::generate_phrases;

fn beats() -> Vec<Ebeat> {
    (0..15).map(|i| Ebeat { time: (i + 1) * 1000, measure: i as i16 }).collect()
}

fn base_arr() -> InstrumentalArrangement {
    InstrumentalArrangement {
        ebeats: beats(),
        levels: vec![Level::default()],
        ..Default::default()
    }
}

#[test]
fn nothing_is_generated_for_arrangement_without_any_levels() {
    let mut arr = InstrumentalArrangement::default();
    generate_phrases(&mut arr);
    assert!(arr.phrases.is_empty());
    assert!(arr.sections.is_empty());
}

#[test]
fn creates_phrases_and_sections_when_one_note() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note { time: 2_000, ..Default::default() });
    arr.levels[0].anchors.push(Anchor { fret: 1, time: 2_000, width: 4, end_time: 0 });
    generate_phrases(&mut arr);
    assert!(!arr.phrases.is_empty());
    assert!(!arr.phrase_iterations.is_empty());
    assert!(!arr.sections.is_empty());
    assert_eq!(arr.phrase_iterations[0].time, 1000);
    assert_eq!(arr.phrase_iterations[1].time, 2000);
    assert_eq!(arr.sections[0].start_time, 2000);
}

#[test]
fn does_not_create_phrase_in_middle_of_note_sustain() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note { time: 2_000, ..Default::default() });
    arr.levels[0].anchors.push(Anchor { fret: 1, time: 2_000, width: 4, end_time: 0 });
    arr.levels[0].notes.push(Note { time: 9_500, sustain: 3_000, ..Default::default() });
    generate_phrases(&mut arr);
    assert!(arr.phrase_iterations.iter().any(|pi| pi.time == 9500));
    assert!(arr.sections.iter().any(|s| s.start_time == 9500));
}

#[test]
fn does_not_create_end_phrase_on_the_last_note() {
    let mut arr = base_arr();
    arr.levels[0].notes.push(Note { time: 2_000, ..Default::default() });
    arr.levels[0].notes.push(Note { time: 6_000, ..Default::default() });
    generate_phrases(&mut arr);
    let end_phrase = arr.phrase_iterations.last().unwrap();
    assert_ne!(end_phrase.time, 6_000);
}
