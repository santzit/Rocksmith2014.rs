use rocksmith2014_xml::ShowLight;
use rocksmith2014_xml_processing::checkers::show_lights_checker::check as check_show_lights;

#[test]
fn detects_missing_fog_note() {
    let sl = vec![ShowLight::new(100, ShowLight::BEAM_MIN)];
    let result = check_show_lights(&sl);
    assert!(result.is_some());
}

#[test]
fn detects_missing_beam_note() {
    let sl = vec![ShowLight::new(100, ShowLight::FOG_MIN)];
    let result = check_show_lights(&sl);
    assert!(result.is_some());
}

#[test]
fn returns_none_for_valid_show_lights() {
    let sl = vec![
        ShowLight::new(100, ShowLight::FOG_MIN),
        ShowLight::new(200, ShowLight::BEAM_OFF),
    ];
    let result = check_show_lights(&sl);
    assert!(result.is_none());
}
