//! Audio file path utilities.
//!
//! Ports `Rocksmith2014.Audio.Utils`.

use crate::error::{AudioError, Result};
use std::path::Path;
use std::time::Duration;

/// Returns the total length of the audio file at the given path.
///
/// Mirrors `Utils.getLength` from the .NET reference.
/// Supports WAV (parsed from RIFF header), OGG Vorbis (via `lewton`),
/// and FLAC (via `claxon`) — matching NAudio's `AudioReader` behaviour.
pub fn get_length(path: &Path) -> Result<Duration> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "wav" => wav_duration(path),
        "ogg" => ogg_duration(path),
        "flac" => flac_duration(path),
        _ => Err(AudioError::Other(format!(
            "get_length: unsupported audio format '{ext}'"
        ))),
    }
}

/// Reads the duration of a PCM WAVE file from its RIFF header.
fn wav_duration(path: &Path) -> Result<Duration> {
    use std::io::{Read, Seek, SeekFrom};

    let mut f = std::fs::File::open(path)?;
    let mut buf4 = [0u8; 4];
    let mut buf2 = [0u8; 2];

    // RIFF header: "RIFF" (4) + file size (4) + "WAVE" (4)
    f.seek(SeekFrom::Start(22))?;
    f.read_exact(&mut buf2)?;
    let num_channels = u16::from_le_bytes(buf2) as u64;

    f.read_exact(&mut buf4)?;
    let sample_rate = u32::from_le_bytes(buf4) as u64;

    f.seek(SeekFrom::Start(34))?;
    f.read_exact(&mut buf2)?;
    let bits_per_sample = u16::from_le_bytes(buf2) as u64;

    // Find the "data" sub-chunk to get its size.
    f.seek(SeekFrom::Start(36))?;
    let mut chunk_id = [0u8; 4];
    let mut chunk_size_buf = [0u8; 4];
    let data_size = loop {
        f.read_exact(&mut chunk_id)?;
        f.read_exact(&mut chunk_size_buf)?;
        let size = u32::from_le_bytes(chunk_size_buf) as u64;
        if &chunk_id == b"data" {
            break size;
        }
        f.seek(SeekFrom::Current(size as i64))?;
    };

    let bytes_per_sample = bits_per_sample / 8;
    if bytes_per_sample == 0 || num_channels == 0 || sample_rate == 0 {
        return Err(AudioError::Other("Invalid WAV header".to_string()));
    }

    let num_samples = data_size / (num_channels * bytes_per_sample);
    let nanos = num_samples * 1_000_000_000 / sample_rate;
    Ok(Duration::from_nanos(nanos))
}

/// Reads the duration of an OGG Vorbis file via the `lewton` crate.
fn ogg_duration(path: &Path) -> Result<Duration> {
    use lewton::inside_ogg::OggStreamReader;

    let f = std::fs::File::open(path)?;
    let mut reader =
        OggStreamReader::new(f).map_err(|e| AudioError::Other(format!("OGG open failed: {e}")))?;

    let sample_rate = reader.ident_hdr.audio_sample_rate as u64;
    let channels = reader.ident_hdr.audio_channels as u64;
    if sample_rate == 0 || channels == 0 {
        return Err(AudioError::Other("Invalid OGG header".to_string()));
    }

    let mut total_samples: u64 = 0;
    while let Some(pck) = reader
        .read_dec_packet_itl()
        .map_err(|e| AudioError::Other(format!("OGG decode failed: {e}")))?
    {
        total_samples += (pck.len() as u64) / channels;
    }

    let nanos = total_samples * 1_000_000_000 / sample_rate;
    Ok(Duration::from_nanos(nanos))
}

/// Reads the duration of a FLAC file via the `claxon` crate.
fn flac_duration(path: &Path) -> Result<Duration> {
    let reader = claxon::FlacReader::open(path)
        .map_err(|e| AudioError::Other(format!("FLAC open failed: {e}")))?;

    let info = reader.streaminfo();
    let sample_rate = info.sample_rate as u64;
    let total_samples = info.samples.unwrap_or(0);

    if sample_rate == 0 {
        return Err(AudioError::Other("Invalid FLAC header".to_string()));
    }

    let nanos = total_samples * 1_000_000_000 / sample_rate;
    Ok(Duration::from_nanos(nanos))
}

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
