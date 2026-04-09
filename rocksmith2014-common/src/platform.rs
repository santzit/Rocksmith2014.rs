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
