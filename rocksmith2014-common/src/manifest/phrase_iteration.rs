/// A phrase iteration in an arrangement manifest.
///
/// Mirrors `PhraseIteration.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhraseIteration {
    pub phrase_index: i32,
    pub max_difficulty: i32,
    pub name: String,
    pub start_time: f32,
    pub end_time: f32,
}
