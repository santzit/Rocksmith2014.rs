//! Manifest JSON serialization/deserialization.
//!
//! Mirrors `Manifest.fs` from `Rocksmith2014.Common`.

use super::attributes::AttributesContainer;
use std::collections::HashMap;
use std::path::Path;

/// The top-level manifest object.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub entries: HashMap<String, AttributesContainer>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub iteration_version: Option<i32>,
    pub insert_root: String,
}

impl Manifest {
    /// Creates a manifest from a single attributes object.
    pub fn create(attrs: super::attributes::Attributes) -> Self {
        let persistent_id = attrs.persistent_id.clone();
        let mut entries = HashMap::new();
        entries.insert(persistent_id, AttributesContainer { attributes: attrs });
        Self {
            entries,
            model_name: Some("RSEnumerable_Song".to_string()),
            iteration_version: Some(2),
            insert_root: "Static.Songs.Entries".to_string(),
        }
    }

    /// Creates a manifest header from multiple attributes objects.
    pub fn create_header(attrs: Vec<super::attributes::Attributes>) -> Self {
        let entries = attrs
            .into_iter()
            .map(|a| {
                let id = a.persistent_id.clone();
                (id, AttributesContainer { attributes: a })
            })
            .collect();
        Self {
            entries,
            model_name: None,
            iteration_version: None,
            insert_root: "Static.Songs.Headers".to_string(),
        }
    }

    /// Returns the single attributes from a manifest.
    pub fn get_singleton_attributes(&self) -> Option<&super::attributes::Attributes> {
        self.entries.values().next().map(|c| &c.attributes)
    }

    /// Serializes the manifest to a JSON string.
    pub fn to_json_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a manifest from a JSON string.
    pub fn from_json_string(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// Deserializes a manifest from a file.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Serializes the manifest to a file.
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }
}
