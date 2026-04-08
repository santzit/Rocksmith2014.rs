//! Tests mirroring Rocksmith2014.DLCProject.Tests/PhraseLevelComparerTests.fs

use rocksmith2014_dlcproject::{
    arrangement::{Arrangement, Instrumental},
    phrase_level_comparer::{compare_levels, ProjectLevels},
};
use rocksmith2014_sng::{Phrase, Sng};
use std::collections::HashMap;

fn make_phrase(name: &str, max_difficulty: i32) -> Phrase {
    let mut raw_name = [0u8; 32];
    let bytes = name.as_bytes();
    raw_name[..bytes.len()].copy_from_slice(bytes);
    Phrase {
        max_difficulty,
        iteration_count: 1,
        name: raw_name,
        ..Default::default()
    }
}

fn test_arrangement() -> (Arrangement, Instrumental) {
    let inst = Instrumental::default();
    let arr = Arrangement::Instrumental(inst.clone());
    (arr, inst)
}

fn stored_levels(inst: &Instrumental) -> ProjectLevels {
    let mut inner = HashMap::new();
    inner.insert("phrase1".to_string(), 5);
    inner.insert("phrase2".to_string(), 2);
    let mut map = HashMap::new();
    map.insert(inst.persistent_id, inner);
    map
}

#[test]
fn detects_phrase_with_fewer_levels_than_stored() {
    let (arr, inst) = test_arrangement();
    let stored = stored_levels(&inst);

    // phrase1 currently has max_difficulty=1, but stored says 5 → regression
    let mut sng = Sng::default();
    sng.phrases = vec![make_phrase("phrase1", 1)];

    let ids = compare_levels(&stored, &[(arr, sng)]);

    assert_eq!(ids.len(), 1, "one ID was returned");
    assert_eq!(ids[0], inst.id, "correct ID was returned");
}

#[test]
fn does_not_return_id_when_levels_have_increased() {
    let (arr, inst) = test_arrangement();
    let stored = stored_levels(&inst);

    // phrase2 currently has max_difficulty=15, stored says 2 → no regression
    let mut sng = Sng::default();
    sng.phrases = vec![make_phrase("phrase2", 15)];

    let ids = compare_levels(&stored, &[(arr, sng)]);

    assert!(ids.is_empty(), "no IDs were returned");
}
