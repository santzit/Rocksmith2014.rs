//! Tests mirroring Rocksmith2014.EOF.Tests/NoteConversionTests.fs

use rocksmith2014_eof::{
    note_converter::convert_notes,
    prepare_notes,
};
use rocksmith2014_xml::{InstrumentalArrangement, Level, MetaData, Note};

#[test]
fn capo_fret_is_reduced_from_notes() {
    let notes = vec![
        Note { time: 100, fret: 0, ..Default::default() },
        Note { time: 200, fret: 5, ..Default::default() },
        Note { time: 300, fret: 8, ..Default::default() },
        Note { time: 300, fret: 7, string: 2, ..Default::default() },
    ];
    let level = Level { notes, ..Default::default() };
    let meta = MetaData { capo: 2, ..Default::default() };
    let inst = InstrumentalArrangement {
        meta,
        levels: vec![level],
        ..Default::default()
    };

    let (raw_notes, _, _) = convert_notes(&inst);
    let prepared = prepare_notes(&[], &inst, raw_notes);

    // Note at time 100: fret 0 — open string stays 0
    assert_eq!(prepared[0].frets[0], 0u8, "1st note fret correct");
    // Note at time 200: fret 5 - capo 2 = 3
    assert_eq!(prepared[1].frets[0], 3u8, "2nd note fret correct");
    // Note at time 300, string 0: fret 8 - capo 2 = 6
    assert_eq!(prepared[2].frets[0], 6u8, "3rd note fret correct");
    // Note at time 300, string 2: fret 7 - capo 2 = 5
    assert_eq!(prepared[2].frets[1], 5u8, "4th note fret correct");
}
