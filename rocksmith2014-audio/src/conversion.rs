//! Audio format conversion utilities.
//!
//! Ports `Rocksmith2014.Audio.Conversion` from the .NET reference implementation.
//!
//! The .NET version uses NAudio for OGG/FLAC → WAV decoding and the
//! `ww2ogg` + `revorb` CLI tools for WEM → OGG conversion.
//! This Rust port uses `lewton` for OGG → WAV, `claxon` for FLAC → WAV,
//! and `hound` to write WAV files.  WEM conversion still uses the same
//! CLI tools (`ww2ogg` + `revorb`) bundled in `Tools/`.

use crate::error::{AudioError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Paths to external tools — mirrors the .NET `toolsDir` convention.
// ---------------------------------------------------------------------------

/// Returns the `Tools/` directory.
///
/// In production the tools are expected next to the executable (matching .NET's
/// `CopyToOutputDirectory` behaviour).  During `cargo test` the exe lives deep
/// inside `target/`, so we fall back to looking for `Tools/` relative to the
/// crate manifest directory so the checked-in binaries are found automatically.
fn tools_dir() -> PathBuf {
    let exe_relative = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_default()
        .join("Tools");

    if exe_relative.is_dir() {
        return exe_relative;
    }

    // Fallback: crate root (baked in at compile time via CARGO_MANIFEST_DIR).
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Tools")
}

/// Platform-aware path to the `ww2ogg` executable inside `tools_dir()`.
fn ww2ogg_path() -> PathBuf {
    tools_dir().join(if cfg!(target_os = "windows") {
        "win/ww2ogg.exe"
    } else if cfg!(target_os = "macos") {
        "mac/ww2ogg"
    } else {
        "linux/ww2ogg"
    })
}

/// Platform-aware path to the `revorb` executable inside `tools_dir()`.
fn revorb_path() -> PathBuf {
    tools_dir().join(if cfg!(target_os = "windows") {
        "win/revorb.exe"
    } else if cfg!(target_os = "macos") {
        "mac/revorb"
    } else {
        "linux/revorb"
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
    let output = Command::new(cmd)
        .arg(path)
        .args(extra_args)
        .current_dir(tools_dir())
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok((exit_code, stdout))
}

/// Validates ww2ogg output.  Errors if exit code is non-zero.
///
/// Mirrors `Conversion.validateWw2oggOutput` from the .NET reference.
fn validate_ww2ogg_output(exit_code: i32, output: &str) -> Result<()> {
    if exit_code != 0 {
        return Err(AudioError::Ww2OggFailed(output.to_string()));
    }
    Ok(())
}

/// Validates revorb output.  Ignores SIGABRT (134) on macOS.
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
    let pcb_str = pcb.to_string_lossy().into_owned();
    let target_str = target_path.to_string_lossy().into_owned();

    let (code, out) = process_file(
        &ww2ogg_path(),
        source_path,
        &["-o", &target_str, "--pcb", &pcb_str],
    )
    .map_err(|e| match e {
        AudioError::Io(io_err) => {
            AudioError::Ww2OggFailed(format!("ww2ogg process failed: {io_err}"))
        }
        other => other,
    })?;
    validate_ww2ogg_output(code, &out)?;

    let (code, out) = process_file(&revorb_path(), target_path, &[]).map_err(|e| match e {
        AudioError::Io(io_err) => AudioError::RevorbFailed(format!("revorb failed: {io_err}")),
        other => other,
    })?;
    validate_revorb_output(code, &out)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Converts a vorbis (OGG) file into a wave file.
///
/// Mirrors `Conversion.oggToWav` from the .NET reference.
/// Uses `lewton` for Vorbis decoding and `hound` for WAV writing.
pub fn ogg_to_wav(source_path: &Path, target_path: &Path) -> Result<()> {
    use lewton::inside_ogg::OggStreamReader;

    let f = std::fs::File::open(source_path)?;
    let mut reader =
        OggStreamReader::new(f).map_err(|e| AudioError::Other(format!("OGG open failed: {e}")))?;

    let spec = hound::WavSpec {
        channels: reader.ident_hdr.audio_channels as u16,
        sample_rate: reader.ident_hdr.audio_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(target_path, spec)
        .map_err(|e| AudioError::Other(format!("WAV create failed: {e}")))?;

    while let Some(samples) = reader
        .read_dec_packet_itl()
        .map_err(|e| AudioError::Other(format!("OGG decode failed: {e}")))?
    {
        for s in samples {
            writer
                .write_sample(s)
                .map_err(|e| AudioError::Other(format!("WAV write failed: {e}")))?;
        }
    }

    writer
        .finalize()
        .map_err(|e| AudioError::Other(format!("WAV finalize failed: {e}")))?;

    Ok(())
}

/// Converts a FLAC file into a wave file.
///
/// Mirrors `Conversion.flacToWav` from the .NET reference.
/// Uses `claxon` for FLAC decoding and `hound` for WAV writing.
pub fn flac_to_wav(source_path: &Path, target_path: &Path) -> Result<()> {
    let mut reader = claxon::FlacReader::open(source_path)
        .map_err(|e| AudioError::Other(format!("FLAC open failed: {e}")))?;

    let info = reader.streaminfo();
    let spec = hound::WavSpec {
        channels: info.channels as u16,
        sample_rate: info.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let bits = info.bits_per_sample;
    let shift = if bits > 16 { bits - 16 } else { 0 };

    let mut writer = hound::WavWriter::create(target_path, spec)
        .map_err(|e| AudioError::Other(format!("WAV create failed: {e}")))?;

    for sample in reader.samples() {
        let raw = sample.map_err(|e| AudioError::Other(format!("FLAC decode failed: {e}")))?;
        let s16 = (raw >> shift).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        writer
            .write_sample(s16)
            .map_err(|e| AudioError::Other(format!("WAV write failed: {e}")))?;
    }

    writer
        .finalize()
        .map_err(|e| AudioError::Other(format!("WAV finalize failed: {e}")))?;

    Ok(())
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
