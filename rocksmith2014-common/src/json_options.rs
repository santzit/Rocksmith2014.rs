/// JSON serialization configuration helpers (serde_json based).
///
/// Mirrors the .NET `JsonOptions` module for controlling serialization settings.
pub struct JsonOptions {
    /// When true, output JSON is pretty-printed.
    pub pretty: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self { pretty: true }
    }
}

impl JsonOptions {
    /// Returns the default options (pretty-printed).
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns compact (non-pretty) options.
    pub fn compact() -> Self {
        Self { pretty: false }
    }
}
