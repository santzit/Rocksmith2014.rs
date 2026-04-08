//! Tests mirroring Rocksmith2014.Audio.Tests/UtilsTests.fs

use rocksmith2014_audio::utils::create_preview_audio_path;

// NOTE: getLength tests (wave/vorbis/flac file length reading) are not yet
// implemented in the Rust crate (no file-based audio decoding).

#[test]
fn preview_audio_path_is_created_from_main_file_path() {
    let path = "C:/path/to/file.ext";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "C:/path/to/file_preview.wav");
}

#[test]
fn preview_audio_path_with_no_directory() {
    let path = "track.ogg";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "track_preview.wav");
}

#[test]
fn preview_audio_path_preserves_stem() {
    let path = "some/path/file.wav";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "some/path/file_preview.wav");
}
