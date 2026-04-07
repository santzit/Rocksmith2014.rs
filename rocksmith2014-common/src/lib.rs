//! Common types shared across Rocksmith 2014 crates.

pub mod compression;

/// The platform for a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Pc,
    Mac,
}

impl Platform {
    /// Returns the audio folder name for the platform.
    pub fn audio_path(&self) -> &'static str {
        match self {
            Platform::Pc => "windows",
            Platform::Mac => "mac",
        }
    }

    /// Returns the SNG folder name for the platform.
    pub fn sng_path(&self) -> &'static str {
        match self {
            Platform::Pc => "generic",
            Platform::Mac => "macos",
        }
    }

    /// Returns the package suffix for the platform.
    pub fn package_suffix(&self) -> &'static str {
        match self {
            Platform::Pc => "_p",
            Platform::Mac => "_m",
        }
    }

    /// Detects platform from a package file name (ending with `_m` means Mac).
    pub fn from_package_file_name(file_path: &str) -> Self {
        let stem = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if stem.ends_with("_m") {
            Platform::Mac
        } else {
            Platform::Pc
        }
    }
}

/// An audio file with a path and volume.
#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path: String,
    pub volume: f64,
}

impl Default for AudioFile {
    fn default() -> Self {
        Self {
            path: String::new(),
            volume: -7.0,
        }
    }
}

/// Random number generation utilities.
pub mod random {
    use std::sync::Mutex;

    static SEED: Mutex<u64> = Mutex::new(12345);

    fn next_u64() -> u64 {
        let mut s = SEED.lock().unwrap();
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    /// Returns a non-negative random integer.
    pub fn next() -> i32 {
        (next_u64() >> 33) as i32
    }

    /// Returns a random integer in `[min, max)`.
    pub fn next_in_range(min: i32, max: i32) -> i32 {
        let range = (max - min) as u64;
        min + (next_u64() % range) as i32
    }

    /// Returns a random lowercase alphabet character.
    pub fn next_alphabet() -> char {
        char::from_u32(b'a' as u32 + (next_u64() % 26) as u32).unwrap_or('a')
    }
}
