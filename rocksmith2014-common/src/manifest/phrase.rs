/// A phrase in an arrangement manifest.
///
/// Mirrors `Phrase.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Phrase {
    pub max_difficulty: i8,
    pub name: String,
    pub iteration_count: i32,
}
