//! Audio utilities for Rocksmith 2014.
//!
//! Ports `Rocksmith2014.Audio` from the .NET reference implementation.
//! Provides pure-Rust implementations of:
//! - BS.1770 / ITU-R R128 LUFS loudness measurement
//! - Preview audio fade parameters
//! - Audio path helpers
//! - Wwise audio conversion (external CLI, not a Rust crate dependency)
//! - WEM / OGG / WAV format conversion

pub mod conversion;
pub mod error;
pub mod fader;
mod iir_filter;
mod lufs_meter;
pub mod preview;
pub mod utils;
pub mod volume;
pub mod wwise;

pub use error::{AudioError, Result};
pub use fader::AudioFader;
pub use preview::PreviewParams;
pub use volume::{calculate_from_file, calculate_lufs};
