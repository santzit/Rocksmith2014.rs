//! Audio file path utilities.
//!
//! Ports `Rocksmith2014.Audio.Utils`.

use std::path::Path;

/// Creates a path for the preview audio file from the given source path.
///
/// # Example
/// ```
/// # use rocksmith2014_audio::utils::create_preview_audio_path;
/// let p = create_preview_audio_path("some/path/file.wav");
/// assert_eq!(p, "some/path/file_preview.wav");
/// ```
pub fn create_preview_audio_path(source_path: &str) -> String {
    let path = Path::new(source_path);
    let dir = path
        .parent()
        .map(|d| d.to_string_lossy())
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    let file_name = format!("{stem}_preview.wav");

    if dir.is_empty() {
        file_name
    } else {
        format!("{dir}/{file_name}")
    }
}
