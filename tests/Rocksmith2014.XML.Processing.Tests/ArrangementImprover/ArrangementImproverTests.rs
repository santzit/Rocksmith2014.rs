use rocksmith2014_xml::{
    Anchor, ArrangementEvent, Chord, ChordMask, ChordNote, ChordTemplate, Ebeat, HandShape,
    InstrumentalArrangement, Level, MetaData, Note, NoteMask, Phrase, PhraseIteration,
};
use rocksmith2014_xml_processing::improvers::custom_events::improve as improve_custom_events;
use rocksmith2014_xml_processing::improvers::improver::{
    add_crowd_events, apply_all_improvements, apply_minimum_improvements, process_chord_names,
    remove_extra_beats,
};

#[test]
fn creates_crowd_events() {
    let notes = vec![Note {
        time: 10000,
        ..Default::default()
    }];
    let level = Level {
        notes,
        ..Default::default()
    };
    let mut arr = InstrumentalArrangement {
        levels: vec![level],
        ..Default::default()
    };
    arr.meta.song_length = 120_000;
    add_crowd_events(&mut arr);
    assert!(!arr.events.is_empty());
}

#[test]
fn no_events_are_created_when_already_present() {
    let notes = vec![Note {
        time: 10000,
        ..Default::default()
    }];
    let level = Level {
        notes,
        ..Default::default()
    };
    let events = vec![
        ArrangementEvent {
            code: "e1".into(),
            time: 1000,
        },
        ArrangementEvent {
            code: "E3".into(),
            time: 10000,
        },
        ArrangementEvent {
            code: "D3".into(),
            time: 20000,
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![level],
        events,
        ..Default::default()
    };
    arr.meta.song_length = 120_000;
    add_crowd_events(&mut arr);
    assert_eq!(arr.events.len(), 3);
}

#[test]
fn removes_beats() {
    let beats = vec![
        Ebeat {
            time: 5000,
            measure: 1,
        },
        Ebeat {
            time: 6000,
            measure: 1,
        },
        Ebeat {
            time: 7000,
            measure: 1,
        },
        Ebeat {
            time: 8000,
            measure: 1,
        },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6000;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 2);
}

#[test]
fn moves_beat_after_end_close_to_it_to_end() {
    let beats = vec![
        Ebeat {
            time: 5000,
            measure: 1,
        },
        Ebeat {
            time: 6000,
            measure: 1,
        },
        Ebeat {
            time: 7000,
            measure: 1,
        },
        Ebeat {
            time: 8000,
            measure: 1,
        },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6900;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 3);
    assert_eq!(arr.ebeats[2].time, 6900);
}

#[test]
fn moves_beat_before_end_close_to_it_to_end() {
    let beats = vec![
        Ebeat {
            time: 5000,
            measure: 1,
        },
        Ebeat {
            time: 6000,
            measure: 1,
        },
        Ebeat {
            time: 7000,
            measure: 1,
        },
        Ebeat {
            time: 8000,
            measure: 1,
        },
    ];
    let mut arr = InstrumentalArrangement {
        ebeats: beats,
        ..Default::default()
    };
    arr.meta.song_length = 6100;
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 2);
    assert_eq!(arr.ebeats[1].time, 6100);
}

#[test]
fn anchor_width_3_event() {
    let anchor = Anchor {
        time: 100,
        fret: 1,
        width: 4,
        end_time: 0,
    };
    let mut arr = InstrumentalArrangement {
        events: vec![ArrangementEvent {
            code: "w3".into(),
            time: 100,
        }],
        levels: vec![Level {
            anchors: vec![anchor],
            ..Default::default()
        }],
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].width, 3);
}

#[test]
fn remove_beats_event() {
    let beats: Vec<_> = (0..5)
        .map(|i| Ebeat {
            time: 100 * (i + 1),
            measure: -1,
        })
        .collect();
    let mut arr = InstrumentalArrangement {
        events: vec![ArrangementEvent {
            code: "removebeats".into(),
            time: 400,
        }],
        ebeats: beats,
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    assert!(arr.ebeats.iter().all(|b| b.time < 400));
}

#[test]
fn fixes_minor_chord_names() {
    let c1 = ChordTemplate {
        name: "Emin".into(),
        display_name: "Emin".into(),
        fingers: [-1; 6],
        frets: [-1; 6],
    };
    let c2 = ChordTemplate {
        name: "Amin7".into(),
        display_name: "Amin7".into(),
        fingers: [-1; 6],
        frets: [-1; 6],
    };
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c1, c2],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert!(!arr.chord_templates[0].name.contains("min"));
    assert!(!arr.chord_templates[1].name.contains("min"));
}

#[test]
fn anchor_width_3_event_can_change_fret() {
    let anchor = Anchor {
        fret: 21,
        time: 180,
        width: 4,
        end_time: 0,
    };
    let mut arr = InstrumentalArrangement {
        events: vec![ArrangementEvent {
            code: "w3-22".into(),
            time: 100,
        }],
        levels: vec![Level {
            anchors: vec![anchor],
            ..Default::default()
        }],
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].width, 3);
    assert_eq!(arr.levels[0].anchors[0].fret, 22);
}

#[test]
fn slide_out_event_works_for_normal_chord() {
    let templates = vec![ChordTemplate {
        name: "".into(),
        display_name: "".into(),
        fingers: [1, 3, -1, -1, -1, -1],
        frets: [1, 3, -1, -1, -1, -1],
    }];
    let cn = vec![
        ChordNote {
            string: 0,
            fret: 1,
            sustain: 1000,
            slide_unpitch_to: 7,
            ..Default::default()
        },
        ChordNote {
            string: 1,
            fret: 3,
            sustain: 1000,
            slide_unpitch_to: 9,
            ..Default::default()
        },
    ];
    let chords = vec![Chord {
        chord_notes: cn,
        ..Default::default()
    }];
    let hs = HandShape {
        chord_id: 0,
        start_time: 0,
        end_time: 1500,
    };
    let mut arr = InstrumentalArrangement {
        chord_templates: templates,
        phrases: vec![Phrase {
            name: "".into(),
            ..Default::default()
        }],
        phrase_iterations: vec![PhraseIteration {
            time: 0,
            phrase_id: 0,
            ..Default::default()
        }],
        events: vec![ArrangementEvent {
            code: "so".into(),
            time: 0,
        }],
        levels: vec![Level {
            chords,
            hand_shapes: vec![hs],
            ..Default::default()
        }],
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    // "so" event should be removed
    assert!(!arr.events.iter().any(|e| e.code == "so"));
    // Handshape end time should be adjusted to sustain (1000)
    assert_eq!(arr.levels[0].hand_shapes[0].end_time, 1000);
}

#[test]
fn slide_out_event_works_for_link_next_chord() {
    let templates = vec![ChordTemplate {
        name: "".into(),
        display_name: "".into(),
        fingers: [-1, -1, 2, 2, -1, -1],
        frets: [-1, -1, 5, 5, -1, -1],
    }];
    let cn = vec![
        ChordNote {
            string: 2,
            fret: 5,
            sustain: 1000,
            mask: NoteMask::LINK_NEXT,
            ..Default::default()
        },
        ChordNote {
            string: 3,
            fret: 5,
            sustain: 1000,
            mask: NoteMask::LINK_NEXT,
            ..Default::default()
        },
    ];
    let chords = vec![Chord {
        chord_notes: cn,
        mask: ChordMask::LINK_NEXT,
        ..Default::default()
    }];
    let notes = vec![
        Note {
            time: 1000,
            string: 2,
            fret: 5,
            sustain: 500,
            slide_unpitch_to: 12,
            ..Default::default()
        },
        Note {
            time: 1000,
            string: 3,
            fret: 5,
            sustain: 500,
            slide_unpitch_to: 12,
            ..Default::default()
        },
    ];
    let hs = HandShape {
        chord_id: 0,
        start_time: 0,
        end_time: 1500,
    };
    let mut arr = InstrumentalArrangement {
        chord_templates: templates,
        phrases: vec![Phrase {
            name: "".into(),
            ..Default::default()
        }],
        phrase_iterations: vec![PhraseIteration {
            time: 0,
            phrase_id: 0,
            ..Default::default()
        }],
        events: vec![ArrangementEvent {
            code: "so".into(),
            time: 1000,
        }],
        levels: vec![Level {
            notes,
            chords,
            hand_shapes: vec![hs],
            ..Default::default()
        }],
        ..Default::default()
    };
    improve_custom_events(&mut arr);
    // "so" event should be removed
    assert!(!arr.events.iter().any(|e| e.code == "so"));
    // Handshape end time should be adjusted to 1000 + 500 = 1500 (note sustain at so_time)
    assert_eq!(arr.levels[0].hand_shapes[0].end_time, 1500);
}

#[test]
fn anchor_before_note_is_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![Anchor {
        fret: 1,
        time: 99,
        width: 4,
        end_time: 0,
    }];
    let notes = vec![Note {
        time: 100,
        fret: 1,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 100);
}

#[test]
fn anchor_after_note_by_5ms_is_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![Anchor {
        fret: 1,
        time: 105,
        width: 4,
        end_time: 0,
    }];
    let notes = vec![Note {
        time: 100,
        fret: 1,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 100);
}

#[test]
fn anchor_after_note_by_6ms_is_not_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![Anchor {
        fret: 1,
        time: 106,
        width: 4,
        end_time: 0,
    }];
    let notes = vec![Note {
        time: 100,
        fret: 1,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 106);
}

#[test]
fn anchor_after_chord_is_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![Anchor {
        fret: 1,
        time: 102,
        width: 4,
        end_time: 0,
    }];
    let chords = vec![Chord {
        time: 100,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            chords,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 100);
}

#[test]
fn anchor_on_note_that_is_very_close_to_another_note_is_not_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![Anchor {
        fret: 1,
        time: 100,
        width: 4,
        end_time: 0,
    }];
    let notes = vec![
        Note {
            time: 100,
            fret: 1,
            ..Default::default()
        },
        Note {
            time: 103,
            fret: 3,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 100);
}

#[test]
fn anchor_at_the_end_of_a_slide_that_is_very_close_to_another_note_is_not_moved() {
    use rocksmith2014_xml_processing::improvers::improver::move_anchors;
    let anchors = vec![
        Anchor {
            fret: 1,
            time: 100,
            width: 4,
            end_time: 0,
        },
        Anchor {
            fret: 3,
            time: 300,
            width: 4,
            end_time: 0,
        },
    ];
    let notes = vec![
        Note {
            time: 100,
            sustain: 200,
            fret: 1,
            slide_to: 3,
            ..Default::default()
        },
        Note {
            time: 303,
            fret: 3,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            anchors,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    move_anchors(&mut arr);
    assert_eq!(arr.levels[0].anchors[0].time, 100);
    assert_eq!(arr.levels[0].anchors[1].time, 300);
}

#[test]
fn extra_anchors_are_not_created_when_moving_phrases() {
    let beats = vec![
        Ebeat {
            time: 900,
            measure: 0,
        },
        Ebeat {
            time: 1000,
            measure: -1,
        },
        Ebeat {
            time: 1200,
            measure: -1,
        },
    ];
    let phrases = vec![
        Phrase {
            name: "mover2".into(),
            ..Default::default()
        },
        Phrase {
            name: "END".into(),
            ..Default::default()
        },
    ];
    let iterations = vec![
        PhraseIteration {
            time: 1000,
            phrase_id: 0,
            ..Default::default()
        },
        PhraseIteration {
            time: 1900,
            phrase_id: 1,
            ..Default::default()
        },
    ];
    let notes = vec![
        Note {
            time: 1000,
            ..Default::default()
        },
        Note {
            time: 1200,
            ..Default::default()
        },
    ];
    let anchors = vec![Anchor {
        fret: 1,
        time: 1200,
        width: 4,
        end_time: 0,
    }];
    let level = Level {
        notes,
        anchors,
        ..Default::default()
    };
    let mut arr = InstrumentalArrangement {
        phrases,
        phrase_iterations: iterations,
        levels: vec![level],
        ebeats: beats,
        meta: MetaData {
            song_length: 2000,
            ..Default::default()
        },
        ..Default::default()
    };
    apply_all_improvements(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 1);
    assert_eq!(arr.levels[0].anchors[0].time, 1200);
}

#[test]
fn removes_notes_without_sustain_after_a_linknext_note() {
    use rocksmith2014_xml_processing::improvers::improver::remove_unnecessary_notes;
    let notes = vec![
        Note {
            time: 1000,
            fret: 5,
            sustain: 500,
            mask: NoteMask::LINK_NEXT,
            ..Default::default()
        },
        Note {
            time: 1500,
            fret: 5,
            sustain: 0,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_unnecessary_notes(&mut arr);
    assert_eq!(arr.levels[0].notes.len(), 1);
    assert_eq!(arr.levels[0].notes[0].time, 1000);
}

#[test]
fn does_not_remove_note_with_sustain_after_a_linknext_note() {
    use rocksmith2014_xml_processing::improvers::improver::remove_unnecessary_notes;
    let notes = vec![
        Note {
            time: 1000,
            fret: 5,
            sustain: 500,
            mask: NoteMask::LINK_NEXT,
            ..Default::default()
        },
        Note {
            time: 1500,
            fret: 5,
            sustain: 200,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_unnecessary_notes(&mut arr);
    assert_eq!(arr.levels[0].notes.len(), 2);
}

#[test]
fn removes_note_without_sustain_after_a_chord() {
    use rocksmith2014_xml_processing::improvers::improver::remove_unnecessary_notes;
    let cn = vec![ChordNote {
        string: 0,
        fret: 5,
        sustain: 500,
        ..Default::default()
    }];
    let chords = vec![Chord {
        time: 1000,
        mask: ChordMask::LINK_NEXT,
        chord_notes: cn,
        ..Default::default()
    }];
    let notes = vec![Note {
        time: 1500,
        string: 0,
        fret: 5,
        sustain: 0,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            chords,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_unnecessary_notes(&mut arr);
    assert_eq!(arr.levels[0].notes.len(), 0);
}

#[test]
fn removes_note_without_sustain_after_a_chord_slide() {
    use rocksmith2014_xml_processing::improvers::improver::remove_unnecessary_notes;
    let cn = vec![ChordNote {
        string: 0,
        fret: 5,
        sustain: 500,
        slide_unpitch_to: 9,
        ..Default::default()
    }];
    let chords = vec![Chord {
        time: 1000,
        chord_notes: cn,
        ..Default::default()
    }];
    let notes = vec![Note {
        time: 1500,
        string: 0,
        fret: 9,
        sustain: 0,
        ..Default::default()
    }];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            chords,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_unnecessary_notes(&mut arr);
    assert_eq!(arr.levels[0].notes.len(), 0);
}

#[test]
fn removes_all_notes_without_sustain_after_a_chord_slide() {
    use rocksmith2014_xml_processing::improvers::improver::remove_unnecessary_notes;
    let cn = vec![
        ChordNote {
            string: 0,
            fret: 5,
            sustain: 500,
            slide_unpitch_to: 9,
            ..Default::default()
        },
        ChordNote {
            string: 1,
            fret: 7,
            sustain: 500,
            slide_unpitch_to: 12,
            ..Default::default()
        },
    ];
    let chords = vec![Chord {
        time: 1000,
        chord_notes: cn,
        ..Default::default()
    }];
    let notes = vec![
        Note {
            time: 1500,
            string: 0,
            fret: 9,
            sustain: 0,
            ..Default::default()
        },
        Note {
            time: 1500,
            string: 1,
            fret: 12,
            sustain: 0,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            chords,
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_unnecessary_notes(&mut arr);
    assert_eq!(arr.levels[0].notes.len(), 0);
}

#[test]
fn removes_harmonic_mask_from_notes() {
    use rocksmith2014_xml_processing::improvers::improver::remove_harmonic_mask;
    let notes = vec![
        Note {
            time: 1000,
            fret: 7,
            slide_to: 9,
            mask: NoteMask::HARMONIC,
            ..Default::default()
        },
        Note {
            time: 1500,
            fret: 7,
            mask: NoteMask::HARMONIC,
            ..Default::default()
        },
    ];
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            notes,
            ..Default::default()
        }],
        ..Default::default()
    };
    remove_harmonic_mask(&mut arr);
    assert!(!arr.levels[0].notes[0].mask.contains(NoteMask::HARMONIC));
    assert!(arr.levels[0].notes[1].mask.contains(NoteMask::HARMONIC));
}

#[test]
fn adds_phrase_start_anchor_from_previous_active_anchor() {
    let mut arr = InstrumentalArrangement {
        phrase_iterations: vec![PhraseIteration {
            time: 1000,
            phrase_id: 0,
            ..Default::default()
        }],
        levels: vec![Level {
            anchors: vec![Anchor {
                time: 900,
                fret: 5,
                width: 3,
                end_time: 0,
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    apply_minimum_improvements(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 2);
    assert_eq!(arr.levels[0].anchors[1].time, 1000);
    assert_eq!(arr.levels[0].anchors[1].fret, 5);
    assert_eq!(arr.levels[0].anchors[1].width, 3);
}

#[test]
fn does_not_add_phrase_start_anchor_when_anchor_already_exists() {
    let mut arr = InstrumentalArrangement {
        phrase_iterations: vec![PhraseIteration {
            time: 1000,
            phrase_id: 0,
            ..Default::default()
        }],
        levels: vec![Level {
            anchors: vec![
                Anchor {
                    time: 900,
                    fret: 5,
                    width: 3,
                    end_time: 0,
                },
                Anchor {
                    time: 1000,
                    fret: 7,
                    width: 4,
                    end_time: 0,
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    apply_minimum_improvements(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 2);
    assert_eq!(arr.levels[0].anchors[1].fret, 7);
}

#[test]
fn does_not_add_phrase_start_anchor_without_previous_anchor() {
    let mut arr = InstrumentalArrangement {
        phrase_iterations: vec![PhraseIteration {
            time: 1000,
            phrase_id: 0,
            ..Default::default()
        }],
        levels: vec![Level {
            anchors: vec![Anchor {
                time: 1200,
                fret: 5,
                width: 3,
                end_time: 0,
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    apply_minimum_improvements(&mut arr);
    assert_eq!(arr.levels[0].anchors.len(), 1);
    assert_eq!(arr.levels[0].anchors[0].time, 1200);
}

#[test]
fn phrase_start_anchor_is_inserted_in_sorted_order() {
    let mut arr = InstrumentalArrangement {
        phrase_iterations: vec![PhraseIteration {
            time: 150,
            phrase_id: 0,
            ..Default::default()
        }],
        levels: vec![Level {
            anchors: vec![
                Anchor {
                    time: 100,
                    fret: 3,
                    width: 4,
                    end_time: 0,
                },
                Anchor {
                    time: 200,
                    fret: 7,
                    width: 4,
                    end_time: 0,
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    apply_minimum_improvements(&mut arr);
    assert_eq!(arr.levels[0].anchors.iter().map(|a| a.time).collect::<Vec<_>>(), vec![100, 150, 200]);
}

#[test]
fn creates_crowd_events_using_start_beat_when_no_notes_or_chords() {
    let mut arr = InstrumentalArrangement {
        meta: MetaData {
            start_beat: 4000,
            song_length: 120_000,
            ..Default::default()
        },
        ..Default::default()
    };
    add_crowd_events(&mut arr);
    assert_eq!(arr.events[0].code, "E3");
    assert_eq!(arr.events[0].time, 4000);
}

#[test]
fn crowd_events_are_inserted_in_sorted_order() {
    let mut arr = InstrumentalArrangement {
        levels: vec![Level {
            notes: vec![Note {
                time: 10_000,
                ..Default::default()
            }],
            ..Default::default()
        }],
        events: vec![ArrangementEvent {
            code: "existing".into(),
            time: 500,
        }],
        meta: MetaData {
            song_length: 120_000,
            ..Default::default()
        },
        ..Default::default()
    };
    add_crowd_events(&mut arr);
    let times: Vec<i32> = arr.events.iter().map(|e| e.time).collect();
    assert!(times.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn remove_extra_beats_does_nothing_when_only_one_beat_exists() {
    let mut arr = InstrumentalArrangement {
        ebeats: vec![Ebeat {
            time: 1000,
            measure: 1,
        }],
        meta: MetaData {
            song_length: 500,
            ..Default::default()
        },
        ..Default::default()
    };
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 1);
    assert_eq!(arr.ebeats[0].time, 1000);
}

#[test]
fn remove_extra_beats_removes_multiple_beats_past_audio_end() {
    let mut arr = InstrumentalArrangement {
        ebeats: vec![
            Ebeat {
                time: 1000,
                measure: 1,
            },
            Ebeat {
                time: 2000,
                measure: 1,
            },
            Ebeat {
                time: 3000,
                measure: 1,
            },
        ],
        meta: MetaData {
            song_length: 1500,
            ..Default::default()
        },
        ..Default::default()
    };
    remove_extra_beats(&mut arr);
    assert_eq!(arr.ebeats.len(), 1);
    assert_eq!(arr.ebeats[0].time, 1500);
}
