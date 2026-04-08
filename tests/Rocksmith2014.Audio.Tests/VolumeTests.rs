//! Tests mirroring Rocksmith2014.Audio.Tests/VolumeTests.fs

use rocksmith2014_audio::volume::calculate_lufs;

// NOTE: File-based Volume.calculate tests (wave/vorbis/flac) are not yet
// implemented in the Rust crate (no file-based audio decoding).

#[test]
fn volume_of_silent_audio_is_zero() {
    // 1 second of stereo silence at 44100 Hz
    let silence = vec![0.0f32; 44100 * 2];
    let vol = calculate_lufs(&silence, 44100.0, 2);
    assert_eq!(vol, 0.0, "Silent audio should produce volume 0.0");
}

#[test]
fn volume_calculation_returns_finite_for_sine_wave() {
    // 440 Hz sine at -18 dBFS, 2 s stereo 44100 Hz
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
    assert_ne!(vol, 0.0, "Volume of sine tone should not be 0.0");
}
