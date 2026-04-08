//! Tests mirroring Rocksmith2014.Audio.Tests/WwiseTests.fs
//!
//! These tests call `rocksmith2014_audio::wwise` functions that are not yet
//! implemented. They will fail to compile until the Wwise CLI integration
//! module is added to `rocksmith2014-audio`.

use rocksmith2014_audio::wwise;

const WAVE_FILE: &str = "BWV0573_wave.wav";
const VORBIS_FILE: &str = "BWV0573_vorbis.ogg";
const FLAC_FILE: &str = "BWV0573_flac.flac";

fn test_conversion(test_file: &std::path::Path) {
    let wem_path = test_file.with_extension("wem");
    if wem_path.exists() {
        std::fs::remove_file(&wem_path).unwrap();
    }

    wwise::convert_to_wem(test_file).unwrap();

    let meta = std::fs::metadata(&wem_path).unwrap();
    assert!(meta.is_file());
    assert!(meta.len() > 100_000);
}

#[test]
fn wave_file_can_be_converted() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_conversion(&manifest_dir.join(WAVE_FILE));
}

#[test]
fn vorbis_file_can_be_converted() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_conversion(&manifest_dir.join(VORBIS_FILE));
}

#[test]
fn flac_file_can_be_converted() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_conversion(&manifest_dir.join(FLAC_FILE));
}

#[test]
fn detection_prioritizes_wwiseroot_environment_variable() {
    let base_dir = std::env::current_dir().unwrap();
    let dummy_wwise_dir = base_dir.join("Wwise Test 2019");
    std::fs::create_dir_all(&dummy_wwise_dir).unwrap();
    let expected_path = dummy_wwise_dir
        .join("Authoring")
        .join("x64")
        .join("Release")
        .join("bin")
        .join("WwiseConsole.exe");
    std::env::set_var("WWISEROOT", &dummy_wwise_dir);

    let cli_path = wwise::find_windows_cli().unwrap();

    assert_eq!(cli_path, expected_path);
}

