use rocksmith2014_dd::comparers::{get_max_similarity_fast, get_same_item_count, same_chord, same_note};
use rocksmith2014_xml::{Chord, ChordMask, Note};

fn n(string: i8, fret: i8) -> Note {
    Note {
        string,
        fret,
        ..Default::default()
    }
}

fn run_same_count(a: &[(i8, i8)], b: &[(i8, i8)]) -> usize {
    let a = a.iter().map(|(s, f)| n(*s, *f)).collect::<Vec<_>>();
    let b = b.iter().map(|(s, f)| n(*s, *f)).collect::<Vec<_>>();
    get_same_item_count(same_note, &a, &b)
}

#[test]
fn correct_similarity_percent_for_two_elements() {
    let l1 = vec![1, 2];
    let l2 = vec![9, 1];
    let similarity = get_max_similarity_fast(|x| *x, &l1, &l2);
    assert_eq!(similarity, 50.0);
}

#[test]
fn correct_similarity_percent_for_three_elements() {
    let l1 = vec![1, 2, 3];
    let l2 = vec![9, 1, 5];
    let similarity = get_max_similarity_fast(|x| *x, &l1, &l2);
    assert!((similarity - 33.33333).abs() < 0.0002);
}

#[test]
fn correct_similarity_percent_for_four_elements() {
    let l1 = vec![1, 2, 3];
    let l2 = vec![9, 1, 5, 3];
    let similarity = get_max_similarity_fast(|x| *x, &l1, &l2);
    assert!((similarity - 66.66666).abs() < 0.0002);
}

#[test]
fn count_is_zero_for_different_notes() {
    assert_eq!(run_same_count(&[(0, 1), (0, 5), (1, 9)], &[(0, 2), (0, 4), (1, 8)]), 0);
}

#[test]
fn correct_for_same_notes() {
    assert_eq!(
        run_same_count(&[(0, 14), (0, 13), (1, 12)], &[(0, 14), (0, 13), (1, 12)]),
        3
    );
}

#[test]
fn correct_for_same_chords() {
    let chords1 = vec![
        Chord {
            chord_id: 50,
            ..Default::default()
        },
        Chord {
            chord_id: 25,
            mask: ChordMask::FRET_HAND_MUTE,
            ..Default::default()
        },
    ];
    let chords2 = vec![
        Chord {
            chord_id: 50,
            ..Default::default()
        },
        Chord {
            chord_id: 25,
            mask: ChordMask::FRET_HAND_MUTE,
            ..Default::default()
        },
    ];
    assert_eq!(get_same_item_count(same_chord, &chords1, &chords2), 2);
}

#[test]
fn correct_when_extra_note_at_the_beginning_1_2() {
    assert_eq!(
        run_same_count(&[(0, 0), (0, 14), (0, 13), (1, 12)], &[(0, 14), (0, 13), (1, 12)]),
        3
    );
}

#[test]
fn correct_when_extra_note_at_the_beginning_2_2() {
    assert_eq!(
        run_same_count(&[(0, 14), (0, 13), (1, 12)], &[(0, 0), (0, 14), (0, 13), (1, 12)]),
        3
    );
}

#[test]
fn correct_when_extra_note_in_between_1_2() {
    assert_eq!(
        run_same_count(&[(0, 14), (0, 13), (1, 15), (1, 12)], &[(0, 14), (0, 13), (1, 12)]),
        3
    );
}

#[test]
fn correct_when_extra_note_in_between_2_2() {
    assert_eq!(
        run_same_count(&[(0, 14), (0, 13), (1, 12)], &[(0, 14), (1, 15), (0, 13), (1, 12)]),
        3
    );
}

#[test]
fn correct_when_two_extra_notes_in_between_one_after_the_other() {
    assert_eq!(
        run_same_count(
            &[(0, 14), (0, 13), (1, 12)],
            &[(0, 14), (1, 15), (1, 15), (0, 13), (1, 12)]
        ),
        3
    );
}

#[test]
fn correct_when_two_extra_notes_in_between_here_and_there() {
    assert_eq!(
        run_same_count(
            &[(0, 14), (5, 22), (1, 13), (5, 22), (1, 12)],
            &[(0, 14), (1, 13), (1, 12)]
        ),
        3
    );
}

#[test]
fn correct_when_two_extra_notes_at_the_beginning() {
    assert_eq!(
        run_same_count(
            &[(0, 0), (0, 0), (0, 14), (0, 13), (1, 12)],
            &[(0, 14), (0, 13), (1, 12)]
        ),
        3
    );
}

#[test]
fn correct_when_starting_note_is_different_1_3() {
    assert_eq!(
        run_same_count(
            &[(0, 10), (0, 14), (0, 13), (1, 12)],
            &[(1, 12), (0, 14), (0, 13), (1, 12)]
        ),
        3
    );
}

#[test]
fn correct_when_starting_note_is_different_2_3() {
    assert_eq!(
        run_same_count(
            &[(1, 7), (0, 5), (0, 4), (0, 5), (0, 4)],
            &[(0, 5), (0, 5), (0, 4), (0, 5), (0, 4)]
        ),
        4
    );
}

#[test]
fn correct_when_starting_note_is_different_3_3() {
    assert_eq!(
        run_same_count(
            &[(0, 5), (0, 5), (0, 4), (0, 5), (0, 4)],
            &[(1, 7), (0, 5), (0, 4), (0, 5), (0, 4)]
        ),
        4
    );
}

#[test]
fn calculation_does_not_take_forever() {
    let mut notes1 = Vec::new();
    let mut notes2 = Vec::new();
    for i in 0..300 {
        notes1.push(n((i % 6) as i8, (i % 12) as i8));
        notes2.push(n(((i + 1) % 6) as i8, (i % 12) as i8));
    }
    let _ = get_same_item_count(same_note, &notes1, &notes2);
    assert!(true);
}

#[test]
fn correct_when_starting_note_is_different_and_one_different_note_in_between() {
    assert_eq!(
        run_same_count(
            &[(0, 4), (0, 1), (0, 8), (0, 8), (0, 8)],
            &[(0, 5), (0, 0), (0, 1), (0, 8), (0, 8), (0, 8)]
        ),
        4
    );
}

#[test]
fn correct_when_6_different_notes_in_between() {
    assert_eq!(
        run_same_count(
            &[(0, 5), (0, 1), (0, 2), (0, 3)],
            &[(0, 5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 1), (0, 2), (0, 3)]
        ),
        4
    );
}

#[test]
fn correct_when_different_starting_and_ending_note_1_2() {
    assert_eq!(
        run_same_count(
            &[(0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 0)],
            &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3)]
        ),
        6
    );
}

#[test]
fn correct_when_different_starting_and_ending_note_2_2() {
    assert_eq!(
        run_same_count(
            &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3)],
            &[(0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 0)]
        ),
        6
    );
}

#[test]
fn correct_when_different_starting_and_ending_note_same_number_of_notes() {
    assert_eq!(
        run_same_count(
            &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 9)],
            &[(0, 9), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 0)]
        ),
        6
    );
}

#[test]
fn correct_when_different_starting_notes_and_ending_note_same_number_of_notes() {
    assert_eq!(
        run_same_count(
            &[(0, 0), (0, 8), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 9)],
            &[(0, 9), (0, 7), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 0)]
        ),
        6
    );
}

#[test]
fn correct_when_different_starting_and_ending_notes_with_different_notes_in_between() {
    assert_eq!(
        run_same_count(
            &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 3), (0, 9)],
            &[
                (0, 9),
                (0, 5),
                (0, 5),
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 0),
            ]
        ),
        6
    );
}

#[test]
fn correct_with_interleaved_same_notes() {
    assert_eq!(
        run_same_count(
            &[(0, 5), (0, 4), (0, 4), (0, 4), (0, 5), (0, 5)],
            &[(0, 4), (0, 5), (0, 5), (0, 5), (0, 4), (0, 4)]
        ),
        3
    );
}
