use crate::arrangement::Arrangement;
use rocksmith2014_common::AudioFile;
use std::path::Path;
use std::time::Duration;

/// A Rocksmith 2014 DLC project.
#[derive(Debug, Clone)]
pub struct DlcProject {
    pub version: String,
    pub author: Option<String>,
    pub dlc_key: String,
    pub artist_name: String,
    pub artist_name_sort: String,
    pub japanese_artist_name: Option<String>,
    pub japanese_title: Option<String>,
    pub title: String,
    pub title_sort: String,
    pub album_name: String,
    pub album_name_sort: String,
    pub year: i32,
    pub album_art_file: String,
    pub audio_file: AudioFile,
    pub audio_file_length: Option<Duration>,
    pub audio_preview_file: AudioFile,
    pub audio_preview_start_time: Option<Duration>,
    pub pitch_shift: Option<i16>,
    pub ignored_issues: Vec<String>,
    pub arrangements: Vec<Arrangement>,
}

impl Default for DlcProject {
    fn default() -> Self {
        DlcProject {
            version: "1".to_string(),
            author: None,
            dlc_key: String::new(),
            artist_name: String::new(),
            artist_name_sort: String::new(),
            japanese_artist_name: None,
            japanese_title: None,
            title: String::new(),
            title_sort: String::new(),
            album_name: String::new(),
            album_name_sort: String::new(),
            year: 2014,
            album_art_file: String::new(),
            audio_file: AudioFile::default(),
            audio_file_length: None,
            audio_preview_file: AudioFile::default(),
            audio_preview_start_time: None,
            pitch_shift: None,
            ignored_issues: Vec::new(),
            arrangements: Vec::new(),
        }
    }
}

/// Returns `true` if the `.wem` version of the audio path needs to be (re)created.
fn needs_conversion(convert_if_newer_than: Duration, path: &str) -> bool {
    // Already a wem file → never convert
    if path.ends_with(".wem") {
        return false;
    }

    let wem_path = Path::new(path).with_extension("wem");

    if !wem_path.exists() {
        // No wem file at all → convert
        return true;
    }

    // Convert if the source file is newer than the existing wem by more than
    // `convert_if_newer_than`.
    let source_modified = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok();
    let wem_modified = std::fs::metadata(&wem_path)
        .and_then(|m| m.modified())
        .ok();

    match (source_modified, wem_modified) {
        (Some(src), Some(wem)) => {
            src.duration_since(wem)
                .map(|diff| diff >= convert_if_newer_than)
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// Returns the paths of all audio files in the project.
fn audio_file_paths<'a>(project: &'a DlcProject) -> impl Iterator<Item = &'a str> {
    let main = std::iter::once(project.audio_file.path.as_str())
        .chain(std::iter::once(project.audio_preview_file.path.as_str()));

    let custom: Vec<&str> = project
        .arrangements
        .iter()
        .filter_map(|arr| {
            if let Arrangement::Instrumental(inst) = arr {
                inst.custom_audio.as_ref().map(|a| a.path.as_str())
            } else {
                None
            }
        })
        .collect();

    main.chain(custom.into_iter())
}

/// Returns the paths to all audio files that need converting to `.wem`.
pub fn get_files_that_need_converting(
    convert_if_newer_than: Duration,
    project: &DlcProject,
) -> Vec<String> {
    audio_file_paths(project)
        .filter(|path| needs_conversion(convert_if_newer_than, path))
        .map(|path| path.to_string())
        .collect()
}
