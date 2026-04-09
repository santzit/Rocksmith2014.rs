//! Attributes DTO for manifest entries.
//!
//! Mirrors `Attributes.fs` from `Rocksmith2014.Common`.

use super::tone::ToneDto as ToneDtoRef;
use super::{ArrangementProperties, ChordTemplate, Phrase, PhraseIteration, Section, Tuning};
use std::collections::HashMap;

/// Full set of manifest attributes for an arrangement.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attributes {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub album_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub album_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub album_name_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub arrangement_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub arrangement_properties: Option<ArrangementProperties>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub arrangement_sort: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub arrangement_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub artist_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub artist_name_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bass_pick: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub block_asset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capo_fret: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cent_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub chords: Option<HashMap<String, HashMap<String, Vec<i32>>>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub chord_templates: Option<Vec<ChordTemplate>>,
    #[serde(default = "bool_true")]
    pub dlc: bool,
    #[serde(default)]
    pub dlc_key: String,
    #[serde(
        rename = "DNA_Chords",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub dna_chords: Option<f64>,
    #[serde(rename = "DNA_Riffs", skip_serializing_if = "Option::is_none", default)]
    pub dna_riffs: Option<f64>,
    #[serde(rename = "DNA_Solo", skip_serializing_if = "Option::is_none", default)]
    pub dna_solo: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dynamic_visual_density: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub easy_mastery: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub input_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub japanese_artist_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub japanese_song_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub japanese_vocal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_conversion_date_time: Option<String>,
    #[serde(default)]
    pub leaderboard_challenge_rating: i32,
    #[serde(default)]
    pub manifest_urn: String,
    #[serde(
        rename = "MasterID_PS3",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub master_id_ps3: Option<i32>,
    #[serde(rename = "MasterID_RDV", default)]
    pub master_id_rdv: i32,
    #[serde(
        rename = "MasterID_XBox360",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub master_id_xbox360: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_phrase_difficulty: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub medium_mastery: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes_easy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes_hard: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes_medium: Option<f32>,
    #[serde(default)]
    pub persistent_id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub phrase_iterations: Option<Vec<PhraseIteration>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub phrases: Option<Vec<Phrase>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preview_bank_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub relative_difficulty: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub representative: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub route_mask: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub score_max_notes: Option<f32>,
    #[serde(rename = "Score_PNV", skip_serializing_if = "Option::is_none", default)]
    pub score_pnv: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sections: Option<Vec<Section>>,
    #[serde(default = "bool_true")]
    pub shipping: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub showlights_xml: Option<String>,
    #[serde(rename = "SKU", default = "default_sku")]
    pub sku: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_asset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_average_tempo: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_bank: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_diff_easy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_diff_hard: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_difficulty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_diff_med: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_event: Option<String>,
    #[serde(default)]
    pub song_key: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_length: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_name_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_offset: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_partition: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_xml: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub song_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub target_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub techniques: Option<HashMap<String, HashMap<String, Vec<i32>>>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_d: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tone_multiplayer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tones: Option<Vec<ToneDtoRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tuning: Option<Tuning>,
}

fn bool_true() -> bool {
    true
}
fn default_sku() -> String {
    "RS2".to_string()
}

/// A wrapper used in the manifest entries map.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttributesContainer {
    pub attributes: Attributes,
}
