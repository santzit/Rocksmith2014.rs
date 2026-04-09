//! Wwise audio conversion utilities.
//!
//! Ports `Rocksmith2014.Audio.Wwise` and `Rocksmith2014.Audio.WwiseFinder`
//! from the .NET reference implementation.
//!
//! Wwise is an external proprietary tool — it is **not** a Rust crate
//! dependency. All conversion work is done by invoking the Wwise CLI
//! (`WwiseConsole.exe` / `WwiseConsole.sh`) as a child process.

pub(crate) mod finder;

use crate::conversion;
use crate::error::{AudioError, Result};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::{result::ZipError, ZipArchive};

// Wwise project templates embedded directly in the binary.
// Mirrors .NET's EmbeddedFileProvider(Assembly.GetExecutingAssembly()).
const WWISE2019_ZIP: &[u8] = include_bytes!("wwise2019.zip");
const WWISE2021_ZIP: &[u8] = include_bytes!("wwise2021.zip");
const WWISE2022_ZIP: &[u8] = include_bytes!("wwise2022.zip");
const WWISE2023_ZIP: &[u8] = include_bytes!("wwise2023.zip");

// Re-export finder functions so callers can use `wwise::find_windows_cli()`.
pub use finder::{find_linux, find_mac, find_windows};

/// Convenience alias matching the test expectation `wwise::find_windows_cli()`.
pub fn find_windows_cli() -> Result<PathBuf> {
    finder::find_windows()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum WwiseVersion {
    Wwise2019,
    Wwise2021,
    Wwise2022,
    Wwise2023,
}

/// Returns the path to the Wwise console executable for the current OS.
///
/// Mirrors `Wwise.getCLIPath` from the .NET reference.
pub fn get_cli_path() -> Result<PathBuf> {
    let path = if cfg!(target_os = "windows") {
        finder::find_windows()?
    } else if cfg!(target_os = "macos") {
        finder::find_mac()?
    } else if cfg!(target_os = "linux") {
        finder::find_linux()?
    } else {
        return Err(AudioError::UnsupportedOs);
    };

    if !path.exists() {
        return Err(AudioError::WwiseNotFound);
    }

    Ok(path)
}

/// Returns an empty temporary directory (created fresh each call).
///
/// Mirrors `Wwise.getTempDirectory` from the .NET reference.
fn get_temp_directory() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(uuid_v4());
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Generates a simple pseudo-random UUID-like string for temp directory names.
/// The .NET version uses `Guid.NewGuid()`.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("wwise-{nanos:x}")
}

/// Determines the Wwise version from the executable path (non-Windows) or
/// its file version metadata (Windows — approximated by path inspection here).
///
/// Mirrors `Wwise.getWwiseVersion` from the .NET reference.
fn get_wwise_version(executable_path: &Path) -> Result<WwiseVersion> {
    let path_str = executable_path.to_string_lossy();
    if path_str.contains("2019") {
        Ok(WwiseVersion::Wwise2019)
    } else if path_str.contains("2021") {
        Ok(WwiseVersion::Wwise2021)
    } else if path_str.contains("2022") {
        Ok(WwiseVersion::Wwise2022)
    } else if path_str.contains("2023") {
        Ok(WwiseVersion::Wwise2023)
    } else {
        // On Windows the .NET version reads FileVersionInfo; here we default
        // to 2021 (matching the .NET fallback for non-Windows paths).
        Ok(WwiseVersion::Wwise2021)
    }
}

fn version_name(version: WwiseVersion) -> &'static str {
    match version {
        WwiseVersion::Wwise2019 => "wwise2019",
        WwiseVersion::Wwise2021 => "wwise2021",
        WwiseVersion::Wwise2022 => "wwise2022",
        WwiseVersion::Wwise2023 => "wwise2023",
    }
}

/// Extracts the embedded Wwise project template for the given version into
/// `target_dir`.
///
/// Mirrors the .NET `extractTemplate` which uses
/// `EmbeddedFileProvider(Assembly.GetExecutingAssembly())` to read the zip
/// from embedded resources.  In Rust we use `include_bytes!()` so the four
/// zip files are compiled directly into the binary — no external files needed.
fn extract_template(target_dir: &Path, version: WwiseVersion) -> Result<()> {
    let zip_bytes: &[u8] = match version {
        WwiseVersion::Wwise2019 => WWISE2019_ZIP,
        WwiseVersion::Wwise2021 => WWISE2021_ZIP,
        WwiseVersion::Wwise2022 => WWISE2022_ZIP,
        WwiseVersion::Wwise2023 => WWISE2023_ZIP,
    };

    // ZipArchive::new accepts any Read + Seek; Cursor<&[u8]> satisfies both.
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e: ZipError| AudioError::Other(format!("Failed to open embedded Wwise template: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e: ZipError| AudioError::Other(e.to_string()))?;
        let out_path = target_dir.join(entry.name());
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    Ok(())
}

/// Extracts the Wwise template and copies the source audio into
/// `Originals/SFX/Audio.wav`, performing any necessary format conversion.
///
/// Mirrors `Wwise.loadTemplate` from the .NET reference.
fn load_template(source_path: &Path, version: WwiseVersion) -> Result<PathBuf> {
    let template_dir = get_temp_directory()?;
    let target_wav = template_dir.join("Originals").join("SFX").join("Audio.wav");

    extract_template(&template_dir, version)?;

    let ext = source_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "wav" => {
            fs::copy(source_path, &target_wav)?;
        }
        "ogg" => {
            conversion::ogg_to_wav(source_path, &target_wav)?;
        }
        "flac" => {
            conversion::flac_to_wav(source_path, &target_wav)?;
        }
        _ => return Err(AudioError::UnknownAudioExtension),
    }

    Ok(template_dir)
}

/// Writes the 4-byte little-endian value `3` at offset 40 of a wem file,
/// making it compatible with Rocksmith 2014.
///
/// Mirrors `Wwise.fixHeader` from the .NET reference.
fn fix_header(path: &Path) -> Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    file.seek(SeekFrom::Start(40))?;
    file.write_all(&3u32.to_le_bytes())?;
    Ok(())
}

/// Locates the generated wem file in the Wwise cache directory and copies it
/// to `dest_path`, then fixes its header.
///
/// Mirrors `Wwise.copyWemFile` from the .NET reference.
fn copy_wem_file(dest_path: &Path, template_dir: &Path) -> Result<()> {
    let cache_path = template_dir.join(".cache").join("Windows").join("SFX");

    let converted = fs::read_dir(&cache_path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|e| e == "wem").unwrap_or(false))
        .ok_or(AudioError::WwiseFileNotFound)?;

    fs::copy(&converted, dest_path)?;
    fix_header(dest_path)?;
    Ok(())
}

/// Builds the `generate-soundbank` argument string for the WwiseConsole CLI.
///
/// Mirrors `Wwise.createArgs` from the .NET reference.
fn create_args(is_linux: bool, template_dir: &Path) -> String {
    let template_path = template_dir.join("Template.wproj");
    let template_str = if is_linux {
        format!("z:{}", template_path.display())
    } else {
        template_path.display().to_string()
    };

    format!(
        r#"generate-soundbank "{template_str}" --platform "Windows" --language "English(US)" --no-decode --quiet"#
    )
}

/// Converts the source audio file into a wem file.
///
/// The destination path is `source_path` with the extension replaced by
/// `.wem`, mirroring `Wwise.convertToWem` from the .NET reference.
///
/// The optional `cli_path` argument mirrors the `cliPath: string option`
/// parameter in the .NET version.  Pass `None` to auto-discover.
pub fn convert_to_wem_with_cli(cli_path: Option<&Path>, source_path: &Path) -> Result<()> {
    let dest_path = source_path.with_extension("wem");

    let cli = match cli_path {
        Some(p) => {
            let s = p.to_string_lossy();
            if !s.contains("WwiseConsole") {
                return Err(AudioError::InvalidCliPath);
            }
            if !p.exists() {
                return Err(AudioError::FileNotFound(s.to_string()));
            }
            p.to_path_buf()
        }
        None => get_cli_path()?,
    };

    let version = get_wwise_version(&cli)?;
    let template_dir = load_template(source_path, version)?;

    let result = (|| -> Result<()> {
        let is_linux = cfg!(target_os = "linux");
        let args = create_args(is_linux, &template_dir);

        let (program, full_args) = if is_linux {
            (
                "wine".to_string(),
                format!("\"{}\" {}", cli.display(), args),
            )
        } else {
            (cli.to_string_lossy().to_string(), args)
        };

        let output = Command::new(&program)
            .args(full_args.split_whitespace())
            .output()?;

        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code != 0 {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let out_msg = if stdout.is_empty() {
                "No console output.".to_string()
            } else {
                format!("Output:\n{stdout}")
            };
            return Err(AudioError::WwiseFailed {
                code: exit_code,
                output: out_msg,
            });
        }

        copy_wem_file(&dest_path, &template_dir)
    })();

    // Always clean up the temp directory, even on error.
    let _ = fs::remove_dir_all(&template_dir);

    result
}

/// Converts the source audio file into a wem file, auto-discovering the
/// Wwise CLI path.
///
/// This is the signature used by `WwiseTests.rs`:
/// `wwise::convert_to_wem(path) -> Result<()>`.
pub fn convert_to_wem(source_path: &Path) -> Result<()> {
    convert_to_wem_with_cli(None, source_path)
}
