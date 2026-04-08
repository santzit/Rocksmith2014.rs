//! Tests mirroring Rocksmith2014.Audio.Tests/PreviewTests.fs

use rocksmith2014_audio::{
    fader::AudioFader,
    preview::{PreviewParams, FADE_IN_MS, FADE_OUT_MS, PREVIEW_LENGTH_MS},
};

// NOTE: File-based Preview.create tests (wave/vorbis/flac) are not yet
// implemented in the Rust crate (no file-based audio decoding).

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
    let short_ms = 4_000u64;
    let params = PreviewParams::for_length(short_ms);
    assert_eq!(params.fade_in_ms, (short_ms / 2) as u32);
    assert_eq!(params.fade_out_ms, (short_ms / 2) as u32);
    assert_eq!(params.preview_length_ms, short_ms);
}

#[test]
fn preview_params_clips_preview_to_preview_length() {
    let params = PreviewParams::for_length(300_000);
    assert_eq!(params.preview_length_ms, PREVIEW_LENGTH_MS);
}

#[test]
fn preview_params_fade_out_start_sample() {
    let params = PreviewParams::for_length(60_000);
    let expected = (PREVIEW_LENGTH_MS - FADE_OUT_MS as u64) * 44100 * 2 / 1000;
    assert_eq!(params.fade_out_start_sample(44100, 2), expected);
}

#[test]
fn fader_first_sample_is_silent_fade_in() {
    let mut fader = AudioFader::new(1000, 500, 5000, 44100, 2);
    let mut buf = vec![1.0f32, 1.0];
    fader.process(&mut buf);
    assert_eq!(buf[0], 0.0);
    assert_eq!(buf[1], 0.0);
}

#[test]
fn fader_last_sample_is_silent_fade_out() {
    let sr = 44100u32;
    let channels = 2usize;
    let audio_ms = 1_000u64;
    let fade_in_ms = 100u32;
    let fade_out_ms = 100u32;

    let mut fader = AudioFader::new(fade_in_ms, fade_out_ms, audio_ms, sr, channels);
    let total_samples = (sr as usize) * channels;
    let mut buf = vec![1.0f32; total_samples];
    fader.process(&mut buf);
    let last = *buf.last().unwrap();
    assert!(
        last <= 0.01,
        "Last sample should be near-zero after fade-out, got {last}"
    );
}
