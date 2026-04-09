/// A section in an arrangement manifest.
///
/// Mirrors `Section.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub name: String,
    pub ui_name: String,
    pub number: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub start_phrase_iteration_index: i32,
    pub end_phrase_iteration_index: i32,
    pub is_solo: bool,
}
