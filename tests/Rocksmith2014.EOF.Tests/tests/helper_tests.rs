use rocksmith2014_eof::helpers::{
    get_closest_beat, infer_time_signatures, try_parse_time_signature,
};
use rocksmith2014_eof::types::EofTimeSignature;
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

    let ts = infer_time_signatures(beats.into_iter());

    let expected = vec![
        (100, EofTimeSignature::Ts3_4),
        (400, EofTimeSignature::Ts2_4),
        (800, EofTimeSignature::Ts4_4),
        (1100, EofTimeSignature::Custom(1, 4)),
    ];
    assert_eq!(ts, expected, "Time signatures should match");
}

#[test]
fn time_signatures_parsed_from_event_string() {
    let inputs = vec![
        ("TS:4/4", Some((4u32, 4u32))),
        ("TS:5/4", Some((5, 4))),
        ("TS:6-8", None),
        ("TS:-1/4", None),
        ("TS:0/2", None),
        ("TS:2/0", None),
    ];
    for (s, expected) in inputs {
        assert_eq!(try_parse_time_signature(s), expected, "parsing {}", s);
    }
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
    let cases = vec![(3900, 0), (4100, 0), (4800, 1), (6750, 3), (8000, 3)];
    for (time, expected) in cases {
        assert_eq!(
            get_closest_beat(&beats, time),
            expected,
            "closest beat for time {}",
            time
        );
    }
}
