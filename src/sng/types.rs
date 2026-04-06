/// SNG data structures mirroring Rocksmith2014.NET v3.5.0.

// ---------------------------------------------------------------------------
// Masks
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct BeatMask: u32 {
        const FIRST_BEAT_OF_MEASURE = 0b01;
        const EVEN_MEASURE          = 0b10;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ChordMask: u32 {
        const ARPEGGIO = 0b01;
        const NOP      = 0b10;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct NoteMask: u32 {
        const CHORD           = 0x0000_0002;
        const OPEN            = 0x0000_0004;
        const FRET_HAND_MUTE  = 0x0000_0008;
        const TREMOLO         = 0x0000_0010;
        const HARMONIC        = 0x0000_0020;
        const PALM_MUTE       = 0x0000_0040;
        const SLAP            = 0x0000_0080;
        const PLUCK           = 0x0000_0100;
        const HAMMER_ON       = 0x0000_0200;
        const PULL_OFF        = 0x0000_0400;
        const SLIDE           = 0x0000_0800;
        const BEND            = 0x0000_1000;
        const SUSTAIN         = 0x0000_2000;
        const TAP             = 0x0000_4000;
        const PINCH_HARMONIC  = 0x0000_8000;
        const VIBRATO         = 0x0001_0000;
        const MUTE            = 0x0002_0000;
        const IGNORE          = 0x0004_0000;
        const LEFT_HAND       = 0x0008_0000;
        const RIGHT_HAND      = 0x0010_0000;
        const HIGH_DENSITY    = 0x0020_0000;
        const UNPITCHED_SLIDE = 0x0040_0000;
        const SINGLE          = 0x0080_0000;
        const CHORD_NOTES     = 0x0100_0000;
        const DOUBLE_STOP     = 0x0200_0000;
        const ACCENT          = 0x0400_0000;
        const PARENT          = 0x0800_0000;
        const CHILD           = 0x1000_0000;
        const ARPEGGIO        = 0x2000_0000;
        const CHORD_PANEL     = 0x8000_0000;
    }
}

// ---------------------------------------------------------------------------
// Leaf structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct BendValue {
    pub time: f32,
    pub step: f32,
    // 4 unknown bytes follow in the file (ignored on read)
}

/// 32-element array of bend values plus a `used_count` field.
#[derive(Debug, Clone)]
pub struct BendData32 {
    pub bend_values: [BendValue; 32],
    pub used_count: i32,
}

impl Default for BendValue {
    fn default() -> Self {
        BendValue { time: 0.0, step: 0.0 }
    }
}

// ---------------------------------------------------------------------------
// Top-level record types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Beat {
    pub time: f32,
    pub measure: i16,
    pub beat: i16,
    pub phrase_iteration: i32,
    pub mask: BeatMask,
}

#[derive(Debug, Clone)]
pub struct Phrase {
    pub solo: i8,
    pub disparity: i8,
    pub ignore: i8,
    pub max_difficulty: i32,
    pub iteration_count: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Chord {
    pub mask: ChordMask,
    pub frets: [i8; 6],
    pub fingers: [i8; 6],
    pub notes: [i32; 6],
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ChordNotes {
    pub mask: [NoteMask; 6],
    pub bend_data: [BendData32; 6],
    pub slide_to: [i8; 6],
    pub slide_unpitch_to: [i8; 6],
    pub vibrato: [i16; 6],
}

#[derive(Debug, Clone)]
pub struct Vocal {
    pub time: f32,
    pub note: i32,
    pub length: f32,
    pub lyric: String,
}

#[derive(Debug, Clone)]
pub struct SymbolsHeader {
    pub id: i32,
    pub unk: [i32; 7],
}

#[derive(Debug, Clone)]
pub struct SymbolsTexture {
    pub font: String,
    pub font_path_length: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub y_min: f32,
    pub x_min: f32,
    pub y_max: f32,
    pub x_max: f32,
}

#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub symbol: String,
    pub outer: Rect,
    pub inner: Rect,
}

#[derive(Debug, Clone)]
pub struct PhraseIteration {
    pub phrase_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub difficulty: [i32; 3],
}

#[derive(Debug, Clone)]
pub struct PhraseExtraInfo {
    pub phrase_id: i32,
    pub difficulty: i32,
    pub empty: i32,
    pub level_jump: i8,
    pub redundant: i16,
}

#[derive(Debug, Clone)]
pub struct NewLinkedDifficulty {
    pub level_break: i32,
    pub nld_phrases: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub time: f32,
    pub action_name: String,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f32,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Tone {
    pub time: f32,
    pub tone_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Dna {
    pub time: f32,
    pub dna_id: i32,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub number: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub start_phrase_iteration_id: i32,
    pub end_phrase_iteration_id: i32,
    pub string_mask: [i8; 36],
}

#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub start_time: f32,
    pub end_time: f32,
    pub first_note_time: f32,
    pub last_note_time: f32,
    pub fret_id: i8,
    pub width: i32,
    pub phrase_iteration_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct AnchorExtension {
    pub beat_time: f32,
    pub fret_id: i8,
}

#[derive(Debug, Clone, Copy)]
pub struct FingerPrint {
    pub chord_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub first_note_time: f32,
    pub last_note_time: f32,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub mask: NoteMask,
    pub flags: u32,
    pub hash: u32,
    pub time: f32,
    pub string_index: i8,
    pub fret: i8,
    pub anchor_fret: i8,
    pub anchor_width: i8,
    pub chord_id: i32,
    pub chord_notes_id: i32,
    pub phrase_id: i32,
    pub phrase_iteration_id: i32,
    pub finger_print_id: [i16; 2],
    pub next_iter_note: i16,
    pub prev_iter_note: i16,
    pub parent_prev_note: i16,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub tap: i8,
    pub pick_direction: i8,
    pub slap: i8,
    pub pluck: i8,
    pub vibrato: i16,
    pub sustain: f32,
    pub max_bend: f32,
    pub bend_data: Vec<BendValue>,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub difficulty: i32,
    pub anchors: Vec<Anchor>,
    pub anchor_extensions: Vec<AnchorExtension>,
    pub hand_shapes: Vec<FingerPrint>,
    pub arpeggios: Vec<FingerPrint>,
    pub notes: Vec<Note>,
    pub average_notes_per_iteration: Vec<f32>,
    pub notes_in_phrase_iterations_excl_ignored: Vec<i32>,
    pub notes_in_phrase_iterations_all: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct MetaData {
    pub max_score: f64,
    pub max_notes_and_chords: f64,
    pub max_notes_and_chords_real: f64,
    pub points_per_note: f64,
    pub first_beat_length: f32,
    pub start_time: f32,
    pub capo_fret_id: i8,
    pub last_conversion_date_time: String,
    pub part: i16,
    pub song_length: f32,
    pub tuning: Vec<i16>,
    pub first_note_time: f32,
    pub max_difficulty: i32,
}

/// The full SNG data structure.
#[derive(Debug, Clone)]
pub struct Sng {
    pub beats: Vec<Beat>,
    pub phrases: Vec<Phrase>,
    pub chords: Vec<Chord>,
    pub chord_notes: Vec<ChordNotes>,
    pub vocals: Vec<Vocal>,
    pub symbols_headers: Vec<SymbolsHeader>,
    pub symbols_textures: Vec<SymbolsTexture>,
    pub symbol_definitions: Vec<SymbolDefinition>,
    pub phrase_iterations: Vec<PhraseIteration>,
    pub phrase_extra_info: Vec<PhraseExtraInfo>,
    pub new_linked_difficulties: Vec<NewLinkedDifficulty>,
    pub actions: Vec<Action>,
    pub events: Vec<Event>,
    pub tones: Vec<Tone>,
    pub dnas: Vec<Dna>,
    pub sections: Vec<Section>,
    pub levels: Vec<Level>,
    pub metadata: MetaData,
}
