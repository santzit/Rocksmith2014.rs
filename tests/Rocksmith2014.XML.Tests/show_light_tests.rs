//! Tests for ShowLight fog/beam ranges and XML serialisation.
//!
//! Mirrors `ShowLightTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::show_light::{self, ShowLight};

#[test]
fn fog_range_test() {
    let mut sl = ShowLight::default();

    for note in ShowLight::FOG_MIN..=ShowLight::FOG_MAX {
        sl.note = note;
        assert!(sl.is_fog(), "note {note} should be fog");
    }

    sl.note = ShowLight::FOG_MAX + 1;
    assert!(!sl.is_fog());

    sl.note = ShowLight::FOG_MIN - 1;
    assert!(!sl.is_fog());
}

#[test]
fn beam_range_test() {
    let mut sl = ShowLight::new(50, ShowLight::FOG_MIN);

    for note in ShowLight::BEAM_MIN..=ShowLight::BEAM_MAX {
        sl.note = note;
        assert!(sl.is_beam(), "note {note} should be beam");
    }

    sl.note = ShowLight::BEAM_OFF;
    assert!(sl.is_beam());

    sl.note = ShowLight::BEAM_MAX + 1;
    assert!(!sl.is_beam());

    sl.note = ShowLight::BEAM_MIN - 1;
    assert!(!sl.is_beam());
}

#[test]
fn list_of_showlights_can_be_saved_to_xml_file() {
    let show_lights = vec![
        ShowLight::new(1_000, ShowLight::BEAM_MIN),
        ShowLight::new(10_000, ShowLight::FOG_MAX),
        ShowLight::new(12_000, ShowLight::BEAM_OFF),
    ];

    let path = std::env::temp_dir().join("showlights_save_test.xml");
    show_light::save(&path, &show_lights).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains("<showlights count=\"3\">"));
    assert!(content.contains("time=\"12.000\" note=\"42\""));
}

#[test]
fn list_of_showlights_can_be_read_from_xml_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("Showlights.xml");

    let show_lights = show_light::load(&path).unwrap();

    assert_eq!(show_lights.len(), 226);

    // <showlight time="18.731" note="35" />
    assert_eq!(show_lights[5].time, 18_731);
    assert_eq!(show_lights[5].note, 35);
}

