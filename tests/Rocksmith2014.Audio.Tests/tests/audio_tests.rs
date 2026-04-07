//! Tests mirroring Rocksmith2014.Audio.Tests (UtilsTests, VolumeTests, PreviewTests).

use rocksmith2014_audio::{
    fader::AudioFader,
    preview::{PreviewParams, FADE_IN_MS, FADE_OUT_MS, PREVIEW_LENGTH_MS},
    utils::create_preview_audio_path,
    volume::calculate_lufs,
};

// ---------------------------------------------------------------------------
// UtilsTests
// ---------------------------------------------------------------------------

#[test]
fn preview_audio_path_is_created_from_main_file_path() {
    let path = "some/path/file.ext";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "some/path/file_preview.wav");
}

#[test]
fn preview_audio_path_with_no_directory() {
    let path = "track.ogg";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "track_preview.wav");
}

#[test]
fn preview_audio_path_preserves_stem() {
    let path = "C:/music/my_song.wav";
    let preview = create_preview_audio_path(path);
    assert_eq!(preview, "C:/music/my_song_preview.wav");
}

// ---------------------------------------------------------------------------
// VolumeTests
// ---------------------------------------------------------------------------

#[test]
fn volume_of_silent_audio_is_zero() {
    // 1 second of stereo silence at 44100 Hz
    let silence = vec![0.0f32; 44100 * 2];
    let vol = calculate_lufs(&silence, 44100.0, 2);
    assert_eq!(vol, 0.0, "Silent audio should produce volume 0.0");
}

#[test]
fn volume_calculation_returns_finite_for_sine_wave() {
    // Generate a 440 Hz sine tone at -18 dBFS (0.125 amplitude), 2 s stereo 44100 Hz
    let sr = 44100usize;
    let dur = 2usize;
    let amp = 0.125f32;
    let mut samples = Vec::with_capacity(sr * dur * 2);
    for i in 0..sr * dur {
        let s = amp * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr as f32).sin();
        samples.push(s);
        samples.push(s);
    }
    let vol = calculate_lufs(&samples, sr as f64, 2);
    assert!(vol.is_finite(), "Volume should be a finite number");
    // For a loud-enough tone the volume should be non-zero
    assert_ne!(vol, 0.0, "Volume of sine tone should not be 0.0");
}

// ---------------------------------------------------------------------------
// PreviewTests
// ---------------------------------------------------------------------------

#[test]
fn preview_params_normal_length() {
    let params = PreviewParams::for_length(60_000);
    assert_eq!(params.fade_in_ms, FADE_IN_MS);
    assert_eq!(params.fade_out_ms, FADE_OUT_MS);
    assert_eq!(params.preview_length_ms, PREVIEW_LENGTH_MS);
}

#[test]
fn preview_params_short_audio_uses_half_length_fades() {
    // Audio shorter than total fade time → each fade = half the audio length
    let short_ms = 4_000u64; // less than 2500 + 3000 = 5500
    let params = PreviewParams::for_length(short_ms);
    assert_eq!(params.fade_in_ms, (short_ms / 2) as u32);
    assert_eq!(params.fade_out_ms, (short_ms / 2) as u32);
    assert_eq!(params.preview_length_ms, short_ms);
}

#[test]
fn preview_params_clips_preview_to_preview_length() {
    // A very long audio file → preview length is capped at PREVIEW_LENGTH_MS
    let params = PreviewParams::for_length(300_000);
    assert_eq!(params.preview_length_ms, PREVIEW_LENGTH_MS);
}

#[test]
fn preview_params_fade_out_start_sample() {
    let params = PreviewParams::for_length(60_000);
    // (28000 - 3000) * 44100 * 2 / 1000 = 25000 * 88.2 = 2_205_000
    let expected = (PREVIEW_LENGTH_MS - FADE_OUT_MS as u64) * 44100 * 2 / 1000;
    assert_eq!(params.fade_out_start_sample(44100, 2), expected);
}

// ---------------------------------------------------------------------------
// AudioFader
// ---------------------------------------------------------------------------

#[test]
fn fader_first_sample_is_silent_fade_in() {
    // With a long fade-in the very first sample frame should be multiplied by ~0
    let mut fader = AudioFader::new(1000, 500, 5000, 44100, 2);
    let mut buf = vec![1.0f32, 1.0]; // one stereo frame
    fader.process(&mut buf);
    // fade_in_pos == 0 at the first frame → multiplier == 0/fade_in_samples == 0
    assert_eq!(buf[0], 0.0);
    assert_eq!(buf[1], 0.0);
}

#[test]
fn fader_last_sample_is_silent_fade_out() {
    // Create a fader for a 1-second clip; submit exactly 1 s of samples so the
    // fade-out should reach its end.
    let sr = 44100u32;
    let channels = 2usize;
    let audio_ms = 1_000u64;
    let fade_in_ms = 100u32;
    let fade_out_ms = 100u32;

    let mut fader = AudioFader::new(fade_in_ms, fade_out_ms, audio_ms, sr, channels);
    let total_samples = (sr as usize) * channels;
    let mut buf = vec![1.0f32; total_samples];
    fader.process(&mut buf);
    // The very last sample frame should be scaled to 0 (fade-out complete)
    let last = *buf.last().unwrap();
    assert!(
        last <= 0.01,
        "Last sample should be near-zero after fade-out, got {last}"
    );
}
