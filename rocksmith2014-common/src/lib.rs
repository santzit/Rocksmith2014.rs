//! Common types shared across Rocksmith 2014 crates.

pub mod binary_readers;
pub mod binary_writers;
pub mod compression;
pub mod json_options;
pub mod memory_stream_pool;
pub mod platform;
pub mod random_generator;
pub mod tone_descriptor;
pub mod types;

pub use platform::Platform;
pub use random_generator::random;
pub use types::AudioFile;
