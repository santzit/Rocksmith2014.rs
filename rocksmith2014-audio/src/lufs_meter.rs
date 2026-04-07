//! BS.1770 / ITU-R R128 integrated loudness (LUFS) meter.
//!
//! Ported from <https://github.com/xuan525/R128Normalization> via the .NET reference.

use crate::iir_filter::SecondOrderIirFilter;

fn channel_weight(channel: usize) -> f64 {
    match channel {
        3 | 4 => 1.41,
        _ => 1.0,
    }
}

fn loudness(mean_square: f64) -> f64 {
    -0.691 + 10.0 * mean_square.log10()
}

/// Mean-square loudness of one 400 ms gating block.
struct BlockLoudness {
    mean_square: f64,
    loudness: f64,
}

/// Integrated (gated) LUFS meter per ITU-R BS.1770-4.
pub struct LufsMeter {
    pre_filter: SecondOrderIirFilter,
    high_pass_filter: SecondOrderIirFilter,
    preceding: Vec<BlockLoudness>,
    // Per-channel rolling step buffer
    step_buffer: Vec<Vec<f64>>,
    step_buffer_pos: usize,
    // Circular buffer of the last `block_step_count` steps
    block_buffer: Vec<Vec<Vec<f64>>>,
    step_sample_count: usize,
    block_step_count: usize,
    num_channels: usize,
}

impl LufsMeter {
    const BLOCK_DURATION: f64 = 0.4; // 400 ms
    const OVERLAP: f64 = 0.75; // 75 % overlap

    /// Creates a new meter for the given sample rate and channel count.
    pub fn new(sample_rate: f64, num_channels: usize) -> Self {
        let block_sample_count = (Self::BLOCK_DURATION * sample_rate).round() as usize;
        let step_sample_count =
            (block_sample_count as f64 * (1.0 - Self::OVERLAP)).round() as usize;
        let block_step_count = block_sample_count / step_sample_count;

        let pre_filter = SecondOrderIirFilter::new(
            1.53512485958697,
            -2.69169618940638,
            1.19839281085285,
            -1.69065929318241,
            0.73248077421585,
            sample_rate,
            num_channels,
        );

        let high_pass_filter = SecondOrderIirFilter::new(
            1.0,
            -2.0,
            1.0,
            -1.99004745483398,
            0.99007225036621,
            sample_rate,
            num_channels,
        );

        let step_buffer = vec![vec![0.0; step_sample_count]; num_channels];
        let block_buffer = vec![vec![vec![0.0; step_sample_count]; num_channels]; block_step_count];

        Self {
            pre_filter,
            high_pass_filter,
            preceding: Vec::new(),
            step_buffer,
            step_buffer_pos: 0,
            block_buffer,
            step_sample_count,
            block_step_count,
            num_channels,
        }
    }

    /// Feeds interleaved `f32` samples (the usual PCM layout) into the meter.
    ///
    /// `samples` is interleaved: `[L0, R0, L1, R1, …]`.
    pub fn process_interleaved(&mut self, samples: &[f32]) {
        let per_channel = samples.len() / self.num_channels;
        let mut buffer: Vec<Vec<f64>> = (0..self.num_channels)
            .map(|ch| {
                (0..per_channel)
                    .map(|i| samples[i * self.num_channels + ch] as f64)
                    .collect()
            })
            .collect();
        self.process_buffer(&mut buffer);
    }

    fn process_buffer(&mut self, buffer: &mut [Vec<f64>]) {
        self.pre_filter.process_buffer(buffer);
        self.high_pass_filter.process_buffer(buffer);

        let buf_len = buffer[0].len();
        let mut buf_pos = 0;

        while buf_pos + (self.step_sample_count - self.step_buffer_pos) <= buf_len {
            let copy_len = self.step_sample_count - self.step_buffer_pos;
            let step_pos = self.step_buffer_pos;
            for (step_ch, buf_ch) in self
                .step_buffer
                .iter_mut()
                .zip(buffer.iter())
                .take(self.num_channels)
            {
                step_ch[step_pos..][..copy_len]
                    .copy_from_slice(&buf_ch[buf_pos..buf_pos + copy_len]);
            }
            buf_pos += copy_len;

            // Rotate the block ring buffer and insert current step.
            self.block_buffer.rotate_left(1);
            let last = self.block_buffer.len() - 1;
            for ch in 0..self.num_channels {
                self.block_buffer[last][ch].copy_from_slice(&self.step_buffer[ch]);
            }
            self.step_buffer_pos = 0;

            // Compute momentary mean-square across the whole 400 ms block.
            let mut momentary_ms = 0.0;
            for ch in 0..self.num_channels {
                let sum_sq: f64 = self
                    .block_buffer
                    .iter()
                    .flat_map(|step| step[ch].iter())
                    .map(|&s| s * s)
                    .sum();
                let mean = sum_sq / (self.block_step_count * self.step_sample_count) as f64;
                momentary_ms += channel_weight(ch) * mean;
            }

            let l = loudness(momentary_ms);
            self.preceding.push(BlockLoudness {
                mean_square: momentary_ms,
                loudness: l,
            });
        }

        // Copy remaining samples into step buffer.
        let remaining = buf_len - buf_pos;
        let step_pos = self.step_buffer_pos;
        for (step_ch, buf_ch) in self
            .step_buffer
            .iter_mut()
            .zip(buffer.iter())
            .take(self.num_channels)
        {
            step_ch[step_pos..][..remaining].copy_from_slice(&buf_ch[buf_pos..]);
        }
        self.step_buffer_pos += remaining;
    }

    /// Returns the integrated LUFS value.
    ///
    /// Returns `f64::NEG_INFINITY` for silent audio.
    pub fn get_integrated_loudness(&self) -> f64 {
        // Absolute gate at −70 LKFS.
        let abs_gated: Vec<&BlockLoudness> = self
            .preceding
            .iter()
            .filter(|b| b.loudness > -70.0)
            .collect();

        if abs_gated.is_empty() {
            return f64::NEG_INFINITY;
        }

        let abs_ms: f64 =
            abs_gated.iter().map(|b| b.mean_square).sum::<f64>() / abs_gated.len() as f64;
        let rel_gate = loudness(abs_ms) - 10.0;

        let rel_gated: Vec<&BlockLoudness> = abs_gated
            .into_iter()
            .filter(|b| b.loudness > rel_gate)
            .collect();

        if rel_gated.is_empty() {
            return f64::NEG_INFINITY;
        }

        let rel_ms: f64 =
            rel_gated.iter().map(|b| b.mean_square).sum::<f64>() / rel_gated.len() as f64;
        loudness(rel_ms)
    }
}
