//! Tests mirroring Rocksmith2014.EOF.Tests/HelperTests.fs

use rocksmith2014_eof::{
    helpers::{get_closest_beat, infer_time_signatures, try_parse_time_signature},
    types::TimeSignature,
};
use rocksmith2014_xml::Ebeat;

#[test]
fn time_signatures_are_calculated_correctly() {
    let beats = vec![
        Ebeat {
            time: 100,
            measure: 0,
        },
        Ebeat {
            time: 200,
            measure: -1,
        },
        Ebeat {
            time: 300,
            measure: -1,
        },
        Ebeat {
            time: 400,
            measure: 1,
        },
        Ebeat {
            time: 500,
            measure: -1,
        },
        Ebeat {
            time: 600,
            measure: 2,
        },
        Ebeat {
            time: 700,
            measure: -1,
        },
        Ebeat {
            time: 800,
            measure: 3,
        },
        Ebeat {
            time: 900,
            measure: -1,
        },
        Ebeat {
            time: 1000,
            measure: -1,
        },
        Ebeat {
            time: 1100,
            measure: -1,
        },
        Ebeat {
            time: 1100,
            measure: 4,
        },
    ];

    let time_signatures = infer_time_signatures(&beats);

    let expected = vec![
        (100, TimeSignature::TS3_4),
        (400, TimeSignature::TS2_4),
        (800, TimeSignature::TS4_4),
        (1100, TimeSignature::Custom(1, 4)),
    ];

    assert_eq!(
        time_signatures, expected,
        "Beat counts and times are correct"
    );
}

#[test]
fn time_signatures_are_parsed_from_event_string_correctly() {
    let ts_strings = ["TS:4/4", "TS:5/4", "TS:6-8", "TS:-1/4", "TS:0/2", "TS:2/0"];

    let time_signatures: Vec<_> = ts_strings
        .iter()
        .map(|s| try_parse_time_signature(s))
        .collect();

    let expected = vec![
        Some((4u32, 4u32)),
        Some((5u32, 4u32)),
        None,
        None,
        None,
        None,
    ];

    assert_eq!(time_signatures, expected, "Parse results were correct");
}

#[test]
fn closest_beat_is_found_correctly() {
    let beats = vec![
        Ebeat {
            time: 4000,
            measure: -1,
        },
        Ebeat {
            time: 5100,
            measure: -1,
        },
        Ebeat {
            time: 6400,
            measure: -1,
        },
        Ebeat {
            time: 7000,
            measure: -1,
        },
    ];
    let times = vec![3900, 4100, 4800, 6750, 8000];

    let closest_beat_numbers: Vec<_> = times.iter().map(|&t| get_closest_beat(&beats, t)).collect();

    let expected = vec![0, 0, 1, 3, 3];

    assert_eq!(closest_beat_numbers, expected, "Closest beats are correct");
}
