/// A chord template with fret and finger positions.
///
/// Mirrors `ChordTemplate.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChordTemplate {
    pub chord_id: i16,
    pub chord_name: String,
    pub fingers: Vec<i8>,
    pub frets: Vec<i8>,
}
