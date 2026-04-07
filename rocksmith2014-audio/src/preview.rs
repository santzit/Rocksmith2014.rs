//! Preview audio parameters and fade-in/fade-out calculation.
//!
//! Ports `Rocksmith2014.Audio.Preview`.

/// Duration of the preview audio in milliseconds.
pub const PREVIEW_LENGTH_MS: u64 = 28_000;

/// Default fade-in duration in milliseconds.
pub const FADE_IN_MS: u32 = 2_500;

/// Default fade-out duration in milliseconds.
pub const FADE_OUT_MS: u32 = 3_000;

/// Resolved preview fade parameters for a specific audio file length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewParams {
    /// Fade-in duration in milliseconds.
    pub fade_in_ms: u32,
    /// Fade-out duration in milliseconds.
    pub fade_out_ms: u32,
    /// Actual preview length in milliseconds (≤ [`PREVIEW_LENGTH_MS`]).
    pub preview_length_ms: u64,
}

impl PreviewParams {
    /// Computes preview fade parameters for an audio file of `audio_length_ms` milliseconds.
    ///
    /// Handles the edge case where the audio is shorter than the total fade time.
    ///
    /// # Example
    /// ```
    /// use rocksmith2014_audio::preview::PreviewParams;
    /// let params = PreviewParams::for_length(60_000);
    /// assert_eq!(params.fade_in_ms, 2_500);
    /// assert_eq!(params.fade_out_ms, 3_000);
    /// assert_eq!(params.preview_length_ms, 28_000);
    /// ```
    pub fn for_length(audio_length_ms: u64) -> Self {
        let total_fade = (FADE_IN_MS + FADE_OUT_MS) as u64;

        let (fade_in_ms, fade_out_ms) = if audio_length_ms < total_fade {
            let half = (audio_length_ms / 2) as u32;
            (half, half)
        } else {
            (FADE_IN_MS, FADE_OUT_MS)
        };

        let preview_length_ms = audio_length_ms.min(PREVIEW_LENGTH_MS);

        Self {
            fade_in_ms,
            fade_out_ms,
            preview_length_ms,
        }
    }

    /// Returns the sample index at which the fade-out begins, given `sample_rate`
    /// and `num_channels`.
    pub fn fade_out_start_sample(&self, sample_rate: u32, num_channels: u32) -> u64 {
        let audio_ms = self.preview_length_ms;
        let fade_out_ms = self.fade_out_ms as u64;
        ((audio_ms - fade_out_ms) * sample_rate as u64 * num_channels as u64) / 1000
    }
}
