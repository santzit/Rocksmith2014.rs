//! Volume (LUFS loudness) calculation.
//!
//! Ports `Rocksmith2014.Audio.Volume`.

use crate::error::{AudioError, Result};
use crate::lufs_meter::LufsMeter;
use std::path::Path;

/// Calculates a volume value using BS.1770 integrated loudness with −16 LUFS as reference.
///
/// Accepts interleaved `f32` PCM samples.  `sample_rate` is in Hz (e.g. 44100)
/// and `num_channels` is typically 2.
///
/// Returns a rounded value suitable for use in a Rocksmith `AudioFile` volume field.
/// Returns `0.0` for silent (infinite-loudness) audio.
///
/// # Example
/// ```
/// use rocksmith2014_audio::volume::calculate_lufs;
/// // Silence → volume rounds to 0.0
/// let silence = vec![0.0f32; 44100 * 2];
/// let vol = calculate_lufs(&silence, 44100.0, 2);
/// assert_eq!(vol, 0.0);
/// ```
pub fn calculate_lufs(samples: &[f32], sample_rate: f64, num_channels: usize) -> f64 {
    let mut meter = LufsMeter::new(sample_rate, num_channels);
    meter.process_interleaved(samples);
    let loudness = meter.get_integrated_loudness();

    if loudness.is_infinite() {
        0.0
    } else {
        let raw = -16.0 - loudness;
        // Round to 1 decimal place with round-half-away-from-zero.
        (raw * 10.0).round() / 10.0
    }
}

// ─── File-level decode helpers ───────────────────────────────────────────────

/// Decode a WAV file to interleaved f32 PCM samples.
fn decode_wav(path: &Path) -> Result<(Vec<f32>, f64, usize)> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| AudioError::Other(format!("WAV open failed: {e}")))?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f64;
    let channels = spec.channels as usize;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AudioError::Other(format!("WAV read failed: {e}")))?,
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample as f32;
            let max_val = (2.0f32).powi(bits as i32 - 1);
            reader
                .samples::<i32>()
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| AudioError::Other(format!("WAV read failed: {e}")))?
                .into_iter()
                .map(|s| s as f32 / max_val)
                .collect()
        }
    };
    Ok((samples, sample_rate, channels))
}

/// Decode an OGG Vorbis file to interleaved f32 PCM samples.
fn decode_ogg(path: &Path) -> Result<(Vec<f32>, f64, usize)> {
    use lewton::inside_ogg::OggStreamReader;
    let f = std::fs::File::open(path)?;
    let mut reader =
        OggStreamReader::new(f).map_err(|e| AudioError::Other(format!("OGG open failed: {e}")))?;
    let sample_rate = reader.ident_hdr.audio_sample_rate as f64;
    let channels = reader.ident_hdr.audio_channels as usize;
    let mut samples = Vec::new();
    while let Some(pkt) = reader
        .read_dec_packet_itl()
        .map_err(|e| AudioError::Other(format!("OGG decode failed: {e}")))?
    {
        for s in pkt {
            samples.push(s as f32 / 32768.0);
        }
    }
    Ok((samples, sample_rate, channels))
}

/// Decode a FLAC file to interleaved f32 PCM samples.
fn decode_flac(path: &Path) -> Result<(Vec<f32>, f64, usize)> {
    let mut reader = claxon::FlacReader::open(path)
        .map_err(|e| AudioError::Other(format!("FLAC open failed: {e}")))?;
    let info = reader.streaminfo();
    let sample_rate = info.sample_rate as f64;
    let channels = info.channels as usize;
    let bits = info.bits_per_sample;
    let max_val = (1i64 << (bits - 1)) as f32;

    let mut samples = Vec::new();
    for s in reader.samples() {
        let raw = s.map_err(|e| AudioError::Other(format!("FLAC decode failed: {e}")))?;
        samples.push(raw as f32 / max_val);
    }
    Ok((samples, sample_rate, channels))
}

/// Decode an audio file (WAV / OGG / FLAC) to interleaved f32 PCM samples.
fn decode_audio_file(path: &Path) -> Result<(Vec<f32>, f64, usize)> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "wav" => decode_wav(path),
        "ogg" => decode_ogg(path),
        "flac" => decode_flac(path),
        other => Err(AudioError::Other(format!(
            "calculate_from_file: unsupported format '{other}'"
        ))),
    }
}

/// Calculates the volume for an audio file.
///
/// Reads WAV, OGG Vorbis, or FLAC files, decodes them to PCM,
/// and returns the rounded volume value matching `Volume.calculate` in
/// the .NET reference implementation.
pub fn calculate_from_file(path: &Path) -> Result<f64> {
    let (samples, sample_rate, channels) = decode_audio_file(path)?;
    Ok(calculate_lufs(&samples, sample_rate, channels))
}
