//! Wwise executable finder.
//!
//! Ports `Rocksmith2014.Audio.WwiseFinder` from the .NET reference implementation.

use crate::error::{AudioError, Result};
use std::path::{Path, PathBuf};

/// Searches `<root_dir>/Audiokinetic/` for a Wwise 2019/2021/2022/2023
/// installation directory and returns its path.
fn try_find_wwise_installation(root_dir: &Path) -> Option<PathBuf> {
    let audiokinetic_dir = root_dir.join("Audiokinetic");
    if !audiokinetic_dir.is_dir() {
        return None;
    }
    std::fs::read_dir(&audiokinetic_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            let name = p.to_string_lossy();
            name.contains("2019")
                || name.contains("2021")
                || name.contains("2022")
                || name.contains("2023")
        })
}

/// Returns the path to the Wwise console executable on Windows.
///
/// Mirrors `WwiseFinder.findWindows` from the .NET reference.
pub fn find_windows() -> Result<PathBuf> {
    let wwise_root = if let Ok(root) = std::env::var("WWISEROOT") {
        let p = PathBuf::from(&root);
        let is_known_version = root.contains("2019")
            || root.contains("2021")
            || root.contains("2022")
            || root.contains("2023");
        if is_known_version && p.is_dir() {
            p
        } else {
            find_from_windows_program_files()?
        }
    } else {
        find_from_windows_program_files()?
    };

    Ok(wwise_root
        .join("Authoring")
        .join("x64")
        .join("Release")
        .join("bin")
        .join("WwiseConsole.exe"))
}

fn find_from_windows_program_files() -> Result<PathBuf> {
    // Try %ProgramFiles(x86)% then %ProgramFiles%
    let candidates = [
        std::env::var("ProgramFiles(x86)").ok(),
        std::env::var("ProgramFiles").ok(),
    ];
    for root in candidates.into_iter().flatten() {
        if let Some(p) = try_find_wwise_installation(Path::new(&root)) {
            return Ok(p);
        }
    }
    Err(AudioError::Other(
        r"Could not locate Wwise 2019/2021/2022/2023 installation from \
          WWISEROOT environment variable or path Program Files\Audiokinetic."
            .to_string(),
    ))
}

/// Returns the path to the Wwise console executable on macOS.
///
/// Mirrors `WwiseFinder.findMac` from the .NET reference.
pub fn find_mac() -> Result<PathBuf> {
    let wwise_app = try_find_wwise_installation(Path::new("/Applications")).ok_or_else(|| {
        AudioError::Other(
            "Could not find Wwise 2019/2021/2022/2023 installation \
                 in /Applications/Audiokinetic/"
                .to_string(),
        )
    })?;
    Ok(wwise_app
        .join("Wwise.app")
        .join("Contents")
        .join("Tools")
        .join("WwiseConsole.sh"))
}

/// Returns the path to the Wwise console executable on Linux (via Wine).
///
/// Mirrors `WwiseFinder.findLinux` from the .NET reference.
pub fn find_linux() -> Result<PathBuf> {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let wine_programs = home
        .join(".wine")
        .join("drive_c")
        .join("Program Files (x86)");
    let wwise_root = try_find_wwise_installation(&wine_programs).ok_or_else(|| {
        AudioError::Other(
            "Could not find Wwise 2019/2021/2022/2023 installation in \
             ~/.wine/drive_c/Program Files (x86)/Audiokinetic/"
                .to_string(),
        )
    })?;
    Ok(wwise_root
        .join("Authoring")
        .join("x64")
        .join("Release")
        .join("bin")
        .join("WwiseConsole.exe"))
}
