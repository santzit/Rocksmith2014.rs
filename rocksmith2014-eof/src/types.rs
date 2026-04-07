use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EofNoteFlag: u32 {
        const HOPO          = 1;
        const CRAZY         = 4;
        const F_HOPO        = 8;
        const ACCENT        = 32;
        const P_HARMONIC    = 64;
        const LINKNEXT      = 128;
        const UNPITCH_SLIDE = 256;
        const HO            = 512;
        const PO            = 1024;
        const TAP           = 2048;
        const SLIDE_UP      = 4096;
        const SLIDE_DOWN    = 8192;
        const STRING_MUTE   = 16384;
        const PALM_MUTE     = 32768;
        const TREMOLO       = 131072;
        const BEND          = 2097152;
        const HARMONIC      = 4194304;
        const VIBRATO       = 16777216;
        const RS_NOTATION   = 33554432;
        const POP           = 67108864;
        const SLAP          = 134217728;
        const SPLIT         = 536870912;
        const EXTENDED_FLAGS = 2147483648;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EofExtendedNoteFlag: u32 {
        const IGNORE     = 1;
        const SUSTAIN    = 2;
        const STOP       = 4;
        const GHOST_HS   = 8;
        const CHORDIFY   = 16;
        const FINGERLESS = 32;
        const PRE_BEND   = 64;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EofTrackFlag: u32 {
        const SIX_LANES       = 1;
        const ALT_NAME        = 2;
        const UNLIMITED_DIFFS = 4;
        const GHL_MODE        = 8;
        const RS_BONUS_ARR    = 16;
        const GHL_MODE_MS     = 32;
        const RS_ALT_ARR      = 64;
        const RS_PICKED_BASS  = 128;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EofEventFlag: u16 {
        const RS_PHRASE      = 1;
        const RS_SECTION     = 2;
        const RS_EVENT       = 4;
        const RS_SOLO_PHRASE = 8;
        const FLOATING_POS   = 16;
    }
}

#[derive(Debug, Clone, Default)]
pub struct EofSection {
    pub name: String,
    pub ty: u8,
    pub start_time: u32,
    pub end_time: u32,
    pub flags: u32,
}

impl EofSection {
    pub fn create(ty: u8, start_time: u32, end_time: u32, flags: u32) -> Self {
        EofSection { name: String::new(), ty, start_time, end_time, flags }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EofNote {
    pub chord_name: String,
    pub chord_number: u8,
    pub difficulty: u8,
    pub bit_flag: u8,
    pub ghost_bit_flag: u8,
    pub frets: Vec<u8>,
    pub legacy_bit_flags: u8,
    pub position: u32,
    pub length: u32,
    pub flags: EofNoteFlag,
    pub slide_end_fret: Option<u8>,
    pub bend_strength: Option<u8>,
    pub unpitched_slide_end_fret: Option<u8>,
    pub extended_note_flags: EofExtendedNoteFlag,
    pub actual_note_position: u32,
    pub end_position: u32,
}

impl EofNote {
    pub fn empty() -> Self {
        EofNote {
            frets: vec![0u8],
            length: 1,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SustainAdjustment {
    pub difficulty: u8,
    pub time: u32,
    pub new_sustain: u32,
}

pub enum HsResult {
    AdjustSustains(Vec<SustainAdjustment>),
    SectionCreated(EofSection),
}

#[derive(Debug, Clone)]
pub struct EofEvent {
    pub text: String,
    pub beat_number: i32,
    pub track_number: u16,
    pub flag: EofEventFlag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IniStringType {
    Custom = 0,
    Artist = 2,
    Title = 3,
    Frettist = 4,
    Year = 6,
    LoadingText = 7,
    Album = 8,
    Genre = 9,
    TrackNumber = 10,
}

#[derive(Debug, Clone)]
pub struct IniString {
    pub string_type: IniStringType,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum TimeSignature {
    TS2_4,
    TS3_4,
    TS4_4,
    TS5_4,
    TS6_4,
    Custom(u32, u32),
}
