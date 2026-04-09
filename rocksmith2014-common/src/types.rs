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
