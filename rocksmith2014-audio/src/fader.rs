//! Audio fade-in / fade-out sample processor.
//!
//! Ports `Rocksmith2014.Audio.AudioFader`.

/// Applies a linear fade-in and fade-out to an interleaved PCM buffer.
///
/// The fader is stateful: call [`AudioFader::process`] with successive
/// chunks of PCM data from the same stream in order.
pub struct AudioFader {
    fade_in_samples: usize,
    fade_out_samples: usize,
    fade_out_start_sample: usize,
    fade_in_pos: usize,
    fade_out_pos: usize,
    total_samples_read: usize,
    num_channels: usize,
}

impl AudioFader {
    /// Creates a new fader.
    ///
    /// * `fade_in_ms`  — length of the fade-in in milliseconds
    /// * `fade_out_ms` — length of the fade-out in milliseconds
    /// * `audio_length_ms` — total length of the clip being faded
    /// * `sample_rate`    — samples per second per channel
    /// * `num_channels`   — channel count
    pub fn new(
        fade_in_ms: u32,
        fade_out_ms: u32,
        audio_length_ms: u64,
        sample_rate: u32,
        num_channels: usize,
    ) -> Self {
        let sr = sample_rate as u64;
        let fade_in_samples = (fade_in_ms as u64 * sr / 1000) as usize;
        let fade_out_samples = (fade_out_ms as u64 * sr / 1000) as usize;
        let fade_out_start_sample =
            ((audio_length_ms - fade_out_ms as u64) * sr * num_channels as u64 / 1000) as usize;

        Self {
            fade_in_samples,
            fade_out_samples,
            fade_out_start_sample,
            fade_in_pos: 0,
            fade_out_pos: 0,
            total_samples_read: 0,
            num_channels,
        }
    }

    /// Applies fading to `buf` (interleaved, any number of frames) in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        let count = buf.len();
        self.total_samples_read += count;

        // Fade-in: apply frame by frame while fade_in_pos ≤ fade_in_samples.
        if self.fade_in_pos <= self.fade_in_samples {
            let mut i = 0;
            while i < count && self.fade_in_pos <= self.fade_in_samples {
                let mult = self.fade_in_pos as f32 / self.fade_in_samples as f32;
                for ch in 0..self.num_channels {
                    if i + ch < count {
                        buf[i + ch] *= mult;
                    }
                }
                i += self.num_channels;
                self.fade_in_pos += 1;
            }
        }

        // Fade-out: starts when total_samples_read crosses fade_out_start_sample.
        if self.total_samples_read >= self.fade_out_start_sample {
            let start_offset = if self.total_samples_read - count < self.fade_out_start_sample {
                let raw = self.fade_out_start_sample - (self.total_samples_read - count);
                // align to channel boundary
                raw - (raw % self.num_channels)
            } else {
                0
            };

            let mut i = start_offset;
            while i < count {
                let mult = 1.0 - (self.fade_out_pos as f32 / self.fade_out_samples.max(1) as f32);
                let mult = mult.max(0.0);
                for ch in 0..self.num_channels {
                    if i + ch < count {
                        buf[i + ch] *= mult;
                    }
                }
                i += self.num_channels;
                self.fade_out_pos += 1;
            }
        }
    }
}
