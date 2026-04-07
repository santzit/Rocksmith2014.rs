use rocksmith2014_common::Platform;

#[test]
fn pc_paths_are_correct() {
    assert_eq!(Platform::Pc.audio_path(), "windows");
    assert_eq!(Platform::Pc.sng_path(), "generic");
    assert_eq!(Platform::Pc.package_suffix(), "_p");
}

#[test]
fn mac_paths_are_correct() {
    assert_eq!(Platform::Mac.audio_path(), "mac");
    assert_eq!(Platform::Mac.sng_path(), "macos");
    assert_eq!(Platform::Mac.package_suffix(), "_m");
}

#[test]
fn detect_pc_from_filename() {
    assert_eq!(
        Platform::from_package_file_name("mysong_p.psarc"),
        Platform::Pc
    );
}

#[test]
fn detect_mac_from_filename() {
    assert_eq!(
        Platform::from_package_file_name("mysong_m.psarc"),
        Platform::Mac
    );
}
