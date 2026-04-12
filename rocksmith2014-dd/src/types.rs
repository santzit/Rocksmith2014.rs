use rocksmith2014_xml::{Chord, HandShape};

#[derive(Debug, Clone)]
pub enum RequestTarget {
    ChordTarget(Chord),
    HandShapeTarget(HandShape),
}

#[derive(Debug, Clone)]
pub struct TemplateRequest {
    pub original_id: i16,
    pub note_count: u8,
    pub from_highest_note: bool,
    pub target: RequestTarget,
}

#[derive(Debug, Clone, Copy)]
pub enum LevelCountGeneration {
    Simple,
    MlModel,
    Constant(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct GeneratorConfig {
    pub phrase_search_threshold: Option<i32>,
    pub level_count_generation: LevelCountGeneration,
}
