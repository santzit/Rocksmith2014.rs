//! Second-order IIR (biquad) filter used for BS.1770 K-weighting.
//!
//! Ported from <https://github.com/xuan525/R128Normalization> via the .NET reference.

/// A second-order IIR (biquad) filter.
///
/// Coefficients are provided at 48 kHz and are resampled automatically when
/// a different sample rate is given.
pub struct SecondOrderIirFilter {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: Vec<f64>,
    z2: Vec<f64>,
    num_channels: usize,
}

impl SecondOrderIirFilter {
    const SAMPLE_RATE_48K: f64 = 48_000.0;

    pub fn new(
        b0_at48k: f64,
        b1_at48k: f64,
        b2_at48k: f64,
        a1_at48k: f64,
        a2_at48k: f64,
        sample_rate: f64,
        num_channels: usize,
    ) -> Self {
        let (b0, b1, b2, a1, a2) = if (sample_rate - Self::SAMPLE_RATE_48K).abs() < 0.5 {
            (b0_at48k, b1_at48k, b2_at48k, a1_at48k, a2_at48k)
        } else {
            // Bilinear transform re-mapping from 48 kHz to the target rate.
            let k_over_q = (2.0 - 2.0 * a2_at48k) / (a2_at48k - a1_at48k + 1.0);
            let k = ((a1_at48k + a2_at48k + 1.0) / (a2_at48k - a1_at48k + 1.0)).sqrt();
            let q = k / k_over_q;
            let arctan_k = k.atan();
            let vb = (b0_at48k - b2_at48k) / (1.0 - a2_at48k);
            let vh = (b0_at48k - b1_at48k + b2_at48k) / (a2_at48k - a1_at48k + 1.0);
            let vl = (b0_at48k + b1_at48k + b2_at48k) / (a1_at48k + a2_at48k + 1.0);

            let k = (arctan_k * Self::SAMPLE_RATE_48K / sample_rate).tan();
            let common = 1.0 / (1.0 + k / q + k * k);
            (
                (vh + vb * k / q + vl * k * k) * common,
                2.0 * (vl * k * k - vh) * common,
                (vh - vb * k / q + vl * k * k) * common,
                2.0 * (k * k - 1.0) * common,
                (1.0 - k / q + k * k) * common,
            )
        };

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            z1: vec![0.0; num_channels],
            z2: vec![0.0; num_channels],
            num_channels,
        }
    }

    /// Processes `buffer[channel][sample]` in-place.
    pub fn process_buffer(&mut self, buffer: &mut [Vec<f64>]) {
        for (ch, channel_buf) in buffer.iter_mut().take(self.num_channels).enumerate() {
            for sample in channel_buf.iter_mut() {
                let factor = *sample - self.a1 * self.z1[ch] - self.a2 * self.z2[ch];
                let out = self.b0 * factor + self.b1 * self.z1[ch] + self.b2 * self.z2[ch];
                self.z2[ch] = self.z1[ch];
                self.z1[ch] = factor;
                *sample = out;
            }
        }
    }
}
