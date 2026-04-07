//! Dynamic difficulty generation for Rocksmith 2014 arrangements.

mod entity_chooser;
mod level_gen;

use rocksmith2014_xml::InstrumentalArrangement;

/// How to determine the number of difficulty levels to generate.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelCountGeneration {
    /// Simple heuristic-based level count.
    Simple,
    /// ML model (not implemented; falls back to Simple).
    MlModel,
    /// Generates exactly this many levels for all phrases.
    Constant(usize),
}

/// Configuration for the DD generator.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Threshold for phrase search similarity (0–100). `None` disables phrase combining.
    pub phrase_search_threshold: Option<i32>,
    /// How to determine the number of difficulty levels.
    pub level_count_generation: LevelCountGeneration,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            phrase_search_threshold: Some(85),
            level_count_generation: LevelCountGeneration::Simple,
        }
    }
}

/// Generates dynamic difficulty levels for the given arrangement in place.
pub fn generate_for_arrangement(config: &GeneratorConfig, arr: &mut InstrumentalArrangement) {
    level_gen::generate(config, arr);
}
