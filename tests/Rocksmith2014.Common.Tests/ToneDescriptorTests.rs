//! Tone descriptor inference tests.
//!
//! Mirrors `ToneDescriptorTests.fs` in Rocksmith2014.Common.Tests (.NET).

use rocksmith2014_common::tone_descriptor::{combine_ui_names, get_descriptions_or_default};

/// test "Default description is 'Clean'"
///
/// Expect.hasLength descriptors 1 "One descriptor exists"
/// Expect.exists descriptors (fun x -> x.Name = "Clean") "Clean descriptor inferred"
#[test]
fn default_description_is_clean() {
    let descriptors = get_descriptions_or_default("tone_test");

    assert_eq!(descriptors.len(), 1, "One descriptor exists");
    assert!(
        descriptors.iter().any(|x| x.name == "Clean"),
        "Clean descriptor inferred"
    );
}

/// test "Can infer a description"
///
/// Expect.hasLength descriptors 1 "One descriptor exists"
/// Expect.exists descriptors (fun x -> x.Name = "Bass") "Bass descriptor inferred"
#[test]
fn can_infer_a_description() {
    let descriptors = get_descriptions_or_default("tone_bass");

    assert_eq!(descriptors.len(), 1, "One descriptor exists");
    assert!(
        descriptors.iter().any(|x| x.name == "Bass"),
        "Bass descriptor inferred"
    );
}

/// test "Can infer a two part description"
///
/// Expect.hasLength descriptors 2 "Two descriptors exist"
/// Expect.exists descriptors (fun x -> x.Name = "Octave") "Octave descriptor inferred"
/// Expect.exists descriptors (fun x -> x.Name = "Lead") "Lead descriptor inferred"
#[test]
fn can_infer_a_two_part_description() {
    let descriptors = get_descriptions_or_default("tone_8valead");

    assert_eq!(descriptors.len(), 2, "Two descriptors exist");
    assert!(
        descriptors.iter().any(|x| x.name == "Octave"),
        "Octave descriptor inferred"
    );
    assert!(
        descriptors.iter().any(|x| x.name == "Lead"),
        "Lead descriptor inferred"
    );
}

/// test "Can infer a three part description"
///
/// Expect.hasLength descriptors 3 "Three descriptors exist"
/// Expect.exists descriptors (fun x -> x.Name = "Rotary") "Rotary descriptor inferred"
/// Expect.exists descriptors (fun x -> x.Name = "Phaser") "Phaser descriptor inferred"
/// Expect.exists descriptors (fun x -> x.Name = "Overdrive") "Overdrive descriptor inferred"
#[test]
fn can_infer_a_three_part_description() {
    let descriptors = get_descriptions_or_default("tone_rotophasedrive");

    assert_eq!(descriptors.len(), 3, "Three descriptors exist");
    assert!(
        descriptors.iter().any(|x| x.name == "Rotary"),
        "Rotary descriptor inferred"
    );
    assert!(
        descriptors.iter().any(|x| x.name == "Phaser"),
        "Phaser descriptor inferred"
    );
    assert!(
        descriptors.iter().any(|x| x.name == "Overdrive"),
        "Overdrive descriptor inferred"
    );
}

/// test "Infers at max three descriptors"
///
/// Expect.hasLength descriptors 3 "Three descriptors exist"
#[test]
fn infers_at_max_three_descriptors() {
    let descriptors = get_descriptions_or_default("tone_rotophasedrivevibtrem");

    assert_eq!(descriptors.len(), 3, "Three descriptors exist");
}

/// test "Can combine UI names"
///
/// ToneDescriptor.getDescriptionsOrDefault "tone_accwah"
/// |> Array.map (fun x -> x.UIName)
/// |> ToneDescriptor.combineUINames
/// Expect.equal res "Acoustic Filter" "Combined name is correct"
#[test]
fn can_combine_ui_names() {
    let descriptors = get_descriptions_or_default("tone_accwah");
    let ui_names: Vec<&str> = descriptors.iter().map(|x| x.ui_name).collect();
    let res = combine_ui_names(&ui_names);

    assert_eq!(res, "Acoustic Filter", "Combined name is correct");
}
