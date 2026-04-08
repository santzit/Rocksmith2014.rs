bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct NoteMask: u16 {
        const LINK_NEXT      = 1 << 0;
        const ACCENT         = 1 << 1;
        const HAMMER_ON      = 1 << 2;
        const HARMONIC       = 1 << 3;
        const IGNORE         = 1 << 4;
        const FRET_HAND_MUTE = 1 << 5;
        const PALM_MUTE      = 1 << 6;
        const PULL_OFF       = 1 << 7;
        const TREMOLO        = 1 << 8;
        const PINCH_HARMONIC = 1 << 9;
        const PICK_DIRECTION = 1 << 10;
        const SLAP           = 1 << 11;
        const PLUCK          = 1 << 12;
        const RIGHT_HAND     = 1 << 13;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ChordMask: u8 {
        const FRET_HAND_MUTE = 1 << 0;
        const HIGH_DENSITY   = 1 << 1;
        const HOPO           = 1 << 2;
        const IGNORE         = 1 << 3;
        const LINK_NEXT      = 1 << 4;
        const PALM_MUTE      = 1 << 5;
        const ACCENT         = 1 << 6;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tuning {
    pub strings: [i16; 6],
}

#[derive(Debug, Clone, Default)]
pub struct ArrangementProperties {
    pub represent: u8,
    pub bonus_arr: u8,
    pub standard_tuning: u8,
    pub non_standard_chords: u8,
    pub barr_chords: u8,
    pub power_chords: u8,
    pub drop_d_power: u8,
    pub open_chords: u8,
    pub finger_picking: u8,
    pub pick_direction: u8,
    pub double_stops: u8,
    pub palm_mutes: u8,
    pub harmonics: u8,
    pub pinch_harmonics: u8,
    pub hopo: u8,
    pub tremolo: u8,
    pub slides: u8,
    pub unpitched_slides: u8,
    pub bends: u8,
    pub tapping: u8,
    pub vibrato: u8,
    pub fret_hand_mutes: u8,
    pub slap_pop: u8,
    pub two_finger_picking: u8,
    pub five_fret_chords: u8,
    pub chord_notes: u8,
    pub octaves: u8,
    pub sus_chords: u8,
    pub three_finger_chords: u8,
    pub rhythm_side: u8,
    pub solo: u8,
    pub path_lead: u8,
    pub path_rhythm: u8,
    pub path_bass: u8,
    pub routing_rules: u8,
    pub bass_pick: u8,
    pub synth_lead: u8,
    pub synth_bass: u8,
}

#[derive(Debug, Clone, Default)]
pub struct MetaData {
    pub song_name: String,
    pub arrangement: String,
    pub part: i32,
    pub offset: i32,
    pub cent_offset: f64,
    pub song_length: i32,
    pub last_conversion_date_time: String,
    pub start_beat: i32,
    pub average_tempo: f64,
    pub tuning: Tuning,
    pub capo: i8,
    pub artist_name: String,
    pub artist_name_sort: String,
    pub album_name: String,
    pub album_name_sort: String,
    pub album_year: i32,
    pub crowd_speed: i32,
    pub arrangement_properties: ArrangementProperties,
    pub tone_base: String,
    pub tone_a: String,
    pub tone_b: String,
    pub tone_c: String,
    pub tone_d: String,
    pub song_name_sort: String,
    pub internal_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Ebeat {
    pub time: i32,
    pub measure: i16,
}

#[derive(Debug, Clone, Default)]
pub struct Phrase {
    pub max_difficulty: u8,
    pub name: String,
    pub disparity: i8,
    pub ignore: i8,
    pub solo: i8,
}

#[derive(Debug, Clone, Default)]
pub struct PhraseIteration {
    pub time: i32,
    pub end_time: i32,
    pub phrase_id: u32,
    pub hero_levels: Option<Vec<HeroLevel>>,
}

#[derive(Debug, Clone, Default)]
pub struct HeroLevel {
    pub hero: i32,
    pub difficulty: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LinkedDiff {
    pub parent_id: i32,
    pub child_id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PhraseProperty {
    pub phrase_id: i32,
    pub redundant: i32,
    pub level_jump: i32,
    pub empty: i32,
    pub difficulty: i32,
}

#[derive(Debug, Clone)]
pub struct ChordTemplate {
    pub chord_name: String,
    pub display_name: String,
    pub fingers: [i8; 6],
    pub frets: [i8; 6],
}
impl Default for ChordTemplate {
    fn default() -> Self {
        Self {
            chord_name: String::new(),
            display_name: String::new(),
            fingers: [-1; 6],
            frets: [-1; 6],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BendValue {
    pub time: i32,
    pub step: f64,
    pub unk5: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Note {
    pub time: i32,
    pub string: i8,
    pub fret: i8,
    pub sustain: i32,
    pub vibrato: i8,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub tap: i8,
    pub pick_direction: i8,
    pub slap: i8,
    pub pluck: i8,
    pub max_bend: f64,
    pub mask: NoteMask,
    pub bend_values: Vec<BendValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ChordNote {
    pub string: i8,
    pub fret: i8,
    pub sustain: i32,
    pub vibrato: i8,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub bend_values: Vec<BendValue>,
    pub mask: NoteMask,
}

#[derive(Debug, Clone, Default)]
pub struct Chord {
    pub time: i32,
    pub chord_id: i32,
    pub sustain: i32,
    pub mask: ChordMask,
    pub chord_notes: Vec<ChordNote>,
}

#[derive(Debug, Clone, Default)]
pub struct Anchor {
    pub time: i32,
    pub end_time: i32,
    pub fret: i8,
    pub width: i32,
}

#[derive(Debug, Clone, Default)]
pub struct HandShape {
    pub chord_id: i32,
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Level {
    pub difficulty: i8,
    pub anchors: Vec<Anchor>,
    pub hand_shapes: Vec<HandShape>,
    pub notes: Vec<Note>,
    pub chords: Vec<Chord>,
}

#[derive(Debug, Clone, Default)]
pub struct ArrangementEvent {
    pub time: i32,
    pub code: String,
}

#[derive(Debug, Clone, Default)]
pub struct Section {
    pub name: String,
    pub number: i32,
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Debug, Clone, Default)]
pub struct ToneChange {
    pub time: i32,
    pub name: String,
    pub id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct InstrumentalArrangement {
    pub meta: MetaData,
    pub ebeats: Vec<Ebeat>,
    pub phrases: Vec<Phrase>,
    pub phrase_iterations: Vec<PhraseIteration>,
    pub linked_diffs: Vec<LinkedDiff>,
    pub phrase_properties: Vec<PhraseProperty>,
    pub chord_templates: Vec<ChordTemplate>,
    pub fret_hand_mute_templates: Vec<ChordTemplate>,
    pub events: Vec<ArrangementEvent>,
    pub sections: Vec<Section>,
    pub levels: Vec<Level>,
    pub tones: Vec<ToneChange>,
}
