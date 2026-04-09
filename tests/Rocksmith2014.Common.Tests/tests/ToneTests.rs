//! Tone XML/JSON import/export tests.
//!
//! Mirrors `ToneTests.fs` in Rocksmith2014.Common.Tests (.NET).

use rocksmith2014_common::manifest::{Gear, Pedal, Tone};
use std::collections::HashMap;

fn test_tone_path() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::PathBuf::from(manifest_dir).join("test.tone2014.xml")
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let tmp = std::env::temp_dir();
    tmp.join(name)
}

/// test "Can be imported from XML"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// Expect.equal tone.Key "Test" "Key is correct"
/// Expect.equal tone.Name "Test" "Name is correct"
/// Expect.equal tone.ToneDescriptors [| "$[35720]CLEAN" |] "Descriptors are correct"
/// Expect.isNone tone.GearList.Cabinet.Category "Cabinet category is none"
/// Expect.isNone tone.GearList.Cabinet.Skin "Cabinet skin is none"
/// Expect.equal tone.GearList.Amp.Key "Amp_OrangeAD50" "Amp key is correct"
/// Expect.hasLength tone.GearList.Amp.KnobValues 4 "There are 4 amp knob values"
/// Expect.isSome tone.GearList.PrePedals.[0] "Pre-pedal 1 was imported"
#[test]
fn can_be_imported_from_xml() {
    let tone = Tone::from_xml_file(test_tone_path()).expect("should parse XML");

    assert_eq!(tone.volume, -12.0, "Volume is correct");
    assert_eq!(tone.key, "Test", "Key is correct");
    assert_eq!(tone.name, "Test", "Name is correct");
    assert_eq!(
        tone.tone_descriptors,
        vec!["$[35720]CLEAN"],
        "Descriptors are correct"
    );
    assert!(
        tone.gear_list.cabinet.category.is_none(),
        "Cabinet category is none"
    );
    assert!(
        tone.gear_list.cabinet.skin.is_none(),
        "Cabinet skin is none"
    );
    assert_eq!(
        tone.gear_list.amp.key, "Amp_OrangeAD50",
        "Amp key is correct"
    );
    assert_eq!(
        tone.gear_list.amp.knob_values.len(),
        4,
        "There are 4 amp knob values"
    );
    assert!(
        tone.gear_list.pre_pedals[0].is_some(),
        "Pre-pedal 1 was imported"
    );
}

/// testTask "Can be exported to XML"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// Expect.equal tone.Key "Test" "Key is correct"
/// Expect.equal tone.Name "Test" "Name is correct"
/// Expect.equal tone.ToneDescriptors [| "$[35720]CLEAN" |] "Descriptors are correct"
/// Expect.equal tone.GearList.Cabinet.Type "Cabinets" "Cabinet type is correct"
/// Expect.isNone tone.GearList.Cabinet.SkinIndex "Cabinet skin index is none"
/// Expect.equal tone.GearList.Amp.Type "Amps" "Amp type is correct"
/// Expect.hasLength tone.GearList.Amp.KnobValues 4 "There are 4 amp knob values"
/// Expect.isSome tone.GearList.PrePedals.[0] "Pre-pedal 1 was imported"
#[test]
fn can_be_exported_to_xml() {
    let test_file = temp_path("testExport.tone2014.xml");
    if test_file.exists() {
        std::fs::remove_file(&test_file).ok();
    }
    let imported = Tone::from_xml_file(test_tone_path()).expect("should import XML");
    Tone::export_xml(&test_file, &imported).expect("should export XML");
    let tone = Tone::from_xml_file(&test_file).expect("should re-import exported XML");

    assert_eq!(tone.volume, -12.0, "Volume is correct");
    assert_eq!(tone.key, "Test", "Key is correct");
    assert_eq!(tone.name, "Test", "Name is correct");
    assert_eq!(
        tone.tone_descriptors,
        vec!["$[35720]CLEAN"],
        "Descriptors are correct"
    );
    assert_eq!(
        tone.gear_list.cabinet.pedal_type, "Cabinets",
        "Cabinet type is correct"
    );
    assert!(
        tone.gear_list.cabinet.skin_index.is_none(),
        "Cabinet skin index is none"
    );
    assert_eq!(tone.gear_list.amp.pedal_type, "Amps", "Amp type is correct");
    assert_eq!(
        tone.gear_list.amp.knob_values.len(),
        4,
        "There are 4 amp knob values"
    );
    assert!(
        tone.gear_list.pre_pedals[0].is_some(),
        "Pre-pedal 1 was imported"
    );
}

/// testTask "Can be exported to JSON and imported from JSON"
///
/// Expect.equal tone.Volume -12. "Volume is correct"
/// Expect.equal tone.Key "Test" "Key is correct"
/// Expect.equal tone.Name "Test" "Name is correct"
/// Expect.equal tone.ToneDescriptors [| "$[35720]CLEAN" |] "Descriptors are correct"
/// Expect.isNone tone.GearList.Cabinet.Skin "Cabinet skin is none"
/// Expect.equal tone.GearList.Amp.Key "Amp_OrangeAD50" "Amp key is correct"
/// Expect.hasLength tone.GearList.Amp.KnobValues 4 "There are 4 amp knob values"
#[test]
fn can_be_exported_to_json_and_imported_from_json() {
    let test_file = temp_path("testExport.tone2014.json");
    if test_file.exists() {
        std::fs::remove_file(&test_file).ok();
    }
    let imported = Tone::from_xml_file(test_tone_path()).expect("should import XML");
    Tone::export_json(&test_file, &imported).expect("should export JSON");
    let tone = Tone::from_json_file(&test_file).expect("should import JSON");

    assert_eq!(tone.volume, -12.0, "Volume is correct");
    assert_eq!(tone.key, "Test", "Key is correct");
    assert_eq!(tone.name, "Test", "Name is correct");
    assert_eq!(
        tone.tone_descriptors,
        vec!["$[35720]CLEAN"],
        "Descriptors are correct"
    );
    assert!(
        tone.gear_list.cabinet.skin.is_none(),
        "Cabinet skin is none"
    );
    assert_eq!(
        tone.gear_list.amp.key, "Amp_OrangeAD50",
        "Amp key is correct"
    );
    assert_eq!(
        tone.gear_list.amp.knob_values.len(),
        4,
        "There are 4 amp knob values"
    );
}

/// test "Number of effects can be counted"
///
/// Expect.equal count 6 "Gear list has 6 effects"
#[test]
fn number_of_effects_can_be_counted() {
    let make_pedal = || Pedal {
        pedal_type: String::new(),
        knob_values: HashMap::new(),
        key: String::new(),
        category: None,
        skin: None,
        skin_index: None,
    };

    let gear = Gear {
        amp: make_pedal(),
        cabinet: make_pedal(),
        racks: [Some(make_pedal()), None, None, Some(make_pedal())],
        pre_pedals: [Some(make_pedal()), None, Some(make_pedal()), None],
        post_pedals: [Some(make_pedal()), Some(make_pedal()), None, None],
    };

    let count = Tone::get_effect_count(&gear);
    assert_eq!(count, 6, "Gear list has 6 effects");
}
