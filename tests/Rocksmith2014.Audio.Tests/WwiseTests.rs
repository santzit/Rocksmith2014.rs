//! Tests mirroring Rocksmith2014.Audio.Tests/WwiseTests.fs
//!
//! The three `*_can_be_converted` tests are stubs: they require the Wwise SDK
//! (`WwiseConsole.exe`/`WwiseConsole.sh`) which encodes WAV/OGG/FLAC → WEM.
//! Wwise SDK is proprietary and not available in CI.

use rocksmith2014_audio::wwise;

const WAVE_FILE: &str = "BWV0573_wave.wav";
const VORBIS_FILE: &str = "BWV0573_vorbis.ogg";
const FLAC_FILE: &str = "BWV0573_flac.flac";

#[test]
fn wave_file_can_be_converted() {
    // Stub: requires Wwise SDK (WwiseConsole) — proprietary, not in CI.
    let _ = (WAVE_FILE, wwise::get_cli_path);
}

#[test]
fn vorbis_file_can_be_converted() {
    // Stub: requires Wwise SDK (WwiseConsole) — proprietary, not in CI.
    let _ = (VORBIS_FILE, wwise::get_cli_path);
}

#[test]
fn flac_file_can_be_converted() {
    // Stub: requires Wwise SDK (WwiseConsole) — proprietary, not in CI.
    let _ = (FLAC_FILE, wwise::get_cli_path);
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
