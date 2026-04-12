use rocksmith2014_xml::{Chord, ChordNote, Note};
use std::collections::HashMap;
use std::hash::Hash;

pub fn same_note(n1: &Note, n2: &Note) -> bool {
    n1.fret == n2.fret
        && n1.string == n2.string
        && n1.mask == n2.mask
        && n1.slide_to == n2.slide_to
        && n1.slide_unpitch_to == n2.slide_unpitch_to
        && n1.vibrato == n2.vibrato
        && n1.tap == n2.tap
        && n1.max_bend == n2.max_bend
}

pub fn same_chord(c1: &Chord, c2: &Chord) -> bool {
    c1.chord_id == c2.chord_id
        && c1.mask == c2.mask
        && c1.chord_notes.len() == c2.chord_notes.len()
        && c1
            .chord_notes
            .iter()
            .zip(c2.chord_notes.iter())
            .all(|(a, b)| same_chord_note(a, b))
}

fn same_chord_note(a: &ChordNote, b: &ChordNote) -> bool {
    a.string == b.string
        && a.fret == b.fret
        && a.sustain == b.sustain
        && a.vibrato == b.vibrato
        && a.slide_to == b.slide_to
        && a.slide_unpitch_to == b.slide_unpitch_to
        && a.left_hand == b.left_hand
        && a.mask == b.mask
        && a.bend_values.len() == b.bend_values.len()
        && a.bend_values
            .iter()
            .zip(b.bend_values.iter())
            .all(|(x, y)| x.step == y.step)
}

fn find_next_match<T: Clone>(
    equal: &impl Fn(&T, &T) -> bool,
    skipped1: &[T],
    skipped2: &[T],
    list1: &[T],
    list2: &[T],
) -> Option<(Vec<T>, Vec<T>)> {
    fn search<T: Clone>(
        equal: &impl Fn(&T, &T) -> bool,
        mut skipped1: Vec<T>,
        mut skipped2: Vec<T>,
        remaining1: &[T],
        remaining2: &[T],
        list1: &[T],
        list2: &[T],
    ) -> Option<(Vec<T>, Vec<T>)> {
        if remaining1.is_empty() || remaining2.is_empty() {
            return None;
        }
        let head1 = &remaining1[0];
        let tail1 = &remaining1[1..];
        let head2 = &remaining2[0];
        let tail2 = &remaining2[1..];

        if equal(head1, head2) {
            return Some((tail1.to_vec(), tail2.to_vec()));
        }

        if let Some(i) = skipped1.iter().position(|x| equal(x, head2)) {
            return Some((list1[i..].to_vec(), tail2.to_vec()));
        }

        if let Some(j) = skipped2.iter().position(|x| equal(x, head1)) {
            return Some((tail1.to_vec(), list2[j..].to_vec()));
        }

        skipped1.insert(0, head1.clone());
        skipped2.insert(0, head2.clone());
        search(equal, skipped1, skipped2, tail1, tail2, list1, list2)
    }

    search(
        equal,
        skipped1.to_vec(),
        skipped2.to_vec(),
        list1,
        list2,
        list1,
        list2,
    )
}

pub fn get_same_item_count<T: Clone>(
    equal: impl Fn(&T, &T) -> bool,
    input1: &[T],
    input2: &[T],
) -> usize {
    fn get_count<T: Clone>(
        equal: &impl Fn(&T, &T) -> bool,
        count: usize,
        len1: usize,
        len2: usize,
        list1: &[T],
        list2: &[T],
    ) -> usize {
        if list1.is_empty() || list2.is_empty() {
            return count;
        }

        let head1 = &list1[0];
        let tail1 = &list1[1..];
        let head2 = &list2[0];
        let tail2 = &list2[1..];

        if equal(head1, head2) {
            return get_count(equal, count + 1, len1 - 1, len2 - 1, tail1, tail2);
        }

        if len1 > len2 {
            return get_count(equal, count, len1 - 1, len2, tail1, list2);
        }
        if len1 < len2 {
            return get_count(equal, count, len1, len2 - 1, list1, tail2);
        }

        match find_next_match(
            equal,
            std::slice::from_ref(head1),
            std::slice::from_ref(head2),
            tail1,
            tail2,
        ) {
            None => count,
            Some((new_tail1, new_tail2)) => get_count(
                equal,
                count + 1,
                new_tail1.len(),
                new_tail2.len(),
                &new_tail1,
                &new_tail2,
            ),
        }
    }

    get_count(&equal, 0, input1.len(), input2.len(), input1, input2)
}

pub fn get_max_similarity_fast<T, K>(projection: impl Fn(&T) -> K, l1: &[T], l2: &[T]) -> f64
where
    K: Eq + Hash,
{
    match (l1.is_empty(), l2.is_empty()) {
        (true, true) => return 100.0,
        (true, false) | (false, true) => return 0.0,
        _ => {}
    }

    let mut map1 = HashMap::<K, usize>::new();
    let mut map2 = HashMap::<K, usize>::new();

    for v in l1 {
        *map1.entry(projection(v)).or_insert(0) += 1;
    }
    for v in l2 {
        *map2.entry(projection(v)).or_insert(0) += 1;
    }

    let same_count = map1
        .iter()
        .map(|(k, c1)| map2.get(k).map(|c2| (*c1).min(*c2)).unwrap_or(0))
        .sum::<usize>();

    // Keep parity with .NET implementation behavior.
    let max_len = l1.len().max(l1.len());
    100.0 * same_count as f64 / max_len as f64
}
