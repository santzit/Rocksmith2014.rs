//! Audio format conversion utilities.
//!
//! Ports `Rocksmith2014.Audio.Conversion` from the .NET reference
//! implementation.
//!
//! The .NET version uses NAudio for OGG/FLAC → WAV decoding and the
//! `ww2ogg` + `revorb` CLI tools for WEM → OGG conversion.
//! This Rust port uses the same CLI tools for the WEM path; OGG/FLAC
//! decoding requires a native audio library and is left as `todo!` until
//! an appropriate crate is added.

use crate::error::{AudioError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Paths to external tools — mirrors the .NET `toolsDir` convention.
// ---------------------------------------------------------------------------

fn tools_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_default()
        .join("Tools")
}

fn ww2ogg_path() -> PathBuf {
    tools_dir().join(if cfg!(target_os = "windows") {
        "ww2ogg.exe"
    } else {
        "ww2ogg"
    })
}

fn revorb_path() -> PathBuf {
    tools_dir().join(if cfg!(target_os = "windows") {
        "revorb.exe"
    } else {
        "revorb"
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Runs `cmd` with `path` as the first argument, plus any `extra_args`.
/// Returns `(exit_code, stdout)`.
///
/// Mirrors `Conversion.processFile` from the .NET reference.
fn process_file(cmd: &Path, path: &Path, extra_args: &[&str]) -> Result<(i32, String)> {
    let mut command = Command::new(cmd);
    command.arg(path).args(extra_args).current_dir(tools_dir());

    let output = command.output()?;
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok((exit_code, stdout))
}

/// Validates ww2ogg output. Errors if exit code is non-zero.
///
/// Mirrors `Conversion.validateWw2oggOutput` from the .NET reference.
fn validate_ww2ogg_output(exit_code: i32, output: &str) -> Result<()> {
    if exit_code != 0 {
        return Err(AudioError::Ww2OggFailed(output.to_string()));
    }
    Ok(())
}

/// Validates revorb output. Ignores SIGABRT (134) on macOS.
///
/// Mirrors `Conversion.validateRevorbOutput` from the .NET reference.
fn validate_revorb_output(exit_code: i32, output: &str) -> Result<()> {
    if cfg!(target_os = "macos") && exit_code == 134 {
        // 134 = SIGABRT – known macOS revorb issue; treat as success.
        // See https://github.com/iminashi/Rocksmith2014.NET/issues/34
        return Ok(());
    }
    if exit_code != 0 {
        let msg = if output.is_empty() {
            format!("exit code: {exit_code}")
        } else {
            format!("output:\n{output}")
        };
        return Err(AudioError::RevorbFailed(msg));
    }
    Ok(())
}

/// Core wem → ogg implementation: runs ww2ogg then revorb.
///
/// Mirrors `Conversion.wemToOggImpl` from the .NET reference.
fn wem_to_ogg_impl(source_path: &Path, target_path: &Path) -> Result<()> {
    let pcb = tools_dir().join("packed_codebooks_aoTuV_603.bin");
    let pcb_str = pcb.to_string_lossy();
    let target_str = target_path.to_string_lossy();

    let (code, out) = process_file(
        &ww2ogg_path(),
        source_path,
        &["-o", &target_str, "--pcb", &pcb_str],
    )?;
    validate_ww2ogg_output(code, &out)?;

    let (code, out) = process_file(&revorb_path(), target_path, &[])?;
    validate_revorb_output(code, &out)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Converts a vorbis (OGG) file into a wave file.
///
/// Mirrors `Conversion.oggToWav` from the .NET reference.
/// The .NET version uses `NAudio.Vorbis.VorbisWaveReader`; a native audio
/// decoding library is needed for a full Rust implementation.
pub fn ogg_to_wav(_source_path: &Path, _target_path: &Path) -> Result<()> {
    todo!(
        "OGG → WAV decoding requires a native audio library \
         (e.g. symphonia). Not yet implemented."
    )
}

/// Converts a FLAC file into a wave file.
///
/// Mirrors `Conversion.flacToWav` from the .NET reference.
pub fn flac_to_wav(_source_path: &Path, _target_path: &Path) -> Result<()> {
    todo!(
        "FLAC → WAV decoding requires a native audio library \
         (e.g. symphonia). Not yet implemented."
    )
}

/// Converts a wem file into a vorbis (OGG) file using `ww2ogg` + `revorb`.
///
/// The output file is placed next to the source with extension `.ogg`.
/// Mirrors `Conversion.wemToOgg` from the .NET reference.
pub fn wem_to_ogg(wem_file: &Path) -> Result<()> {
    let ogg_file = wem_file.with_extension("ogg");
    wem_to_ogg_impl(wem_file, &ogg_file)
}

/// Converts a wem file into a wave file (via an intermediate OGG step).
///
/// Mirrors `Conversion.wemToWav` from the .NET reference.
pub fn wem_to_wav(wem_file: &Path) -> Result<()> {
    wem_to_ogg(wem_file)?;

    let ogg_file = wem_file.with_extension("ogg");
    let wav_file = wem_file.with_extension("wav");
    ogg_to_wav(&ogg_file, &wav_file)?;
    std::fs::remove_file(&ogg_file)?;
    Ok(())
}

/// Performs `f` on a temporary OGG file converted from `wem_path`, then
/// deletes the temporary file.
///
/// Mirrors `Conversion.withTempOggFile` from the .NET reference.
pub fn with_temp_ogg_file<F, T>(f: F, wem_path: &Path) -> Result<T>
where
    F: FnOnce(&Path) -> T,
{
    let temp_ogg = std::env::temp_dir().join(format!(
        "{}.ogg",
        wem_path.file_stem().unwrap_or_default().to_string_lossy()
    ));
    wem_to_ogg_impl(wem_path, &temp_ogg)?;
    let result = f(&temp_ogg);
    let _ = std::fs::remove_file(&temp_ogg);
    Ok(result)
}
