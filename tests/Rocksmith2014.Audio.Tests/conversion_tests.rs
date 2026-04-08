//! Tests mirroring Rocksmith2014.Audio.Tests/ConversionTests.fs
//!
//! These tests call `rocksmith2014_audio::conversion` functions that are not
//! yet implemented. They will fail to compile until the conversion module
//! (ww2ogg-based ogg↔wav and wem→ogg) is added to `rocksmith2014-audio`.

use rocksmith2014_audio::conversion;
use rocksmith2014_audio::utils;

const VORBIS_FILE: &str = "BWV0573_vorbis.ogg";
const WEM_FILE: &str = "BWV0573_wwise.wem";

#[test]
fn vorbis_file_can_be_converted_to_wave_file() {
    let target_file = std::env::temp_dir().join("convtest.wav");
    if target_file.exists() {
        std::fs::remove_file(&target_file).unwrap();
    }
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ogg_path = manifest_dir.join(VORBIS_FILE);
    let ogg_length = utils::get_length(&ogg_path).unwrap();

    conversion::ogg_to_wav(&ogg_path, &target_file).unwrap();
    let wav_length = utils::get_length(&target_file).unwrap();

    assert_eq!(wav_length, ogg_length);
}

#[test]
fn wem_file_can_be_converted_to_vorbis_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wem_path = manifest_dir.join(WEM_FILE);
    let target_file = wem_path.with_extension("ogg");
    if target_file.exists() {
        std::fs::remove_file(&target_file).unwrap();
    }

    conversion::wem_to_ogg(&wem_path).unwrap();
    let ogg_length = utils::get_length(&target_file).unwrap();

    assert_eq!(ogg_length.as_secs(), 42);
}

#[test]
fn conversion_throws_error_on_missing_file() {
    let wem_file = std::path::Path::new("nosuchfile.wem");
    let result = conversion::wem_to_ogg(wem_file);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("ww2ogg process failed"));
}

