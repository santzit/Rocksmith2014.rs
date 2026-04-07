//! Volume (LUFS loudness) calculation.
//!
//! Ports `Rocksmith2014.Audio.Volume`.

use crate::lufs_meter::LufsMeter;

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
