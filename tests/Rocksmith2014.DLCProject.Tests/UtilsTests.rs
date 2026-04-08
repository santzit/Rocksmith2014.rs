//! Tests mirroring Rocksmith2014.DLCProject.Tests/UtilsTests.fs

use rocksmith2014_dlcproject::utils::{get_tuning_name, TuningName};

#[test]
fn correct_tuning_name_for_standard_tuning() {
    let e_standard = [0i16; 6];
    let name = get_tuning_name(&e_standard);
    assert_eq!(
        name,
        TuningName::Translatable("Standard".to_string(), vec!["E".to_string()])
    );
}

#[test]
fn correct_tuning_name_for_dadgad() {
    let dadgad = [-2i16, 0, 0, 0, -2, -2];
    let name = get_tuning_name(&dadgad);
    assert_eq!(name, TuningName::Custom("DADGAD".to_string()));
}

#[test]
fn correct_tuning_name_for_custom_tuning() {
    let custom = [-1i16, 1, -12, 0, 2, -13];
    let name = get_tuning_name(&custom);
    assert_eq!(name, TuningName::Custom("EbBbDGDbEb".to_string()));
}
