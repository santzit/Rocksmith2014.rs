//! Audio utilities for Rocksmith 2014.
//!
//! Ports `Rocksmith2014.Audio` from the .NET reference implementation.
//! Provides pure-Rust implementations of:
//! - BS.1770 / ITU-R R128 LUFS loudness measurement
//! - Preview audio fade parameters
//! - Audio path helpers

pub mod fader;
mod iir_filter;
mod lufs_meter;
pub mod preview;
pub mod utils;
pub mod volume;

pub use fader::AudioFader;
pub use preview::PreviewParams;
pub use volume::calculate_lufs;
