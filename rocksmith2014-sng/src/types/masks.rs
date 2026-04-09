bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BeatMask: i32 {
        const FIRST_BEAT_OF_MEASURE = 0b01;
        const EVEN_MEASURE          = 0b10;
    }
}

impl Default for BeatMask {
    fn default() -> Self {
        BeatMask::empty()
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NoteMask: u32 {
        const CHORD           = 0x00000002;
        const OPEN            = 0x00000004;
        const FRET_HAND_MUTE  = 0x00000008;
        const TREMOLO         = 0x00000010;
        const HARMONIC        = 0x00000020;
        const PALM_MUTE       = 0x00000040;
        const SLAP            = 0x00000080;
        const PLUCK           = 0x00000100;
        const HAMMER_ON       = 0x00000200;
        const PULL_OFF        = 0x00000400;
        const SLIDE           = 0x00000800;
        const BEND            = 0x00001000;
        const SUSTAIN         = 0x00002000;
        const TAP             = 0x00004000;
        const PINCH_HARMONIC  = 0x00008000;
        const VIBRATO         = 0x00010000;
        const MUTE            = 0x00020000;
        const IGNORE          = 0x00040000;
        const LEFT_HAND       = 0x00080000;
        const RIGHT_HAND      = 0x00100000;
        const HIGH_DENSITY    = 0x00200000;
        const UNPITCHED_SLIDE = 0x00400000;
        const SINGLE          = 0x00800000;
        const CHORD_NOTES     = 0x01000000;
        const DOUBLE_STOP     = 0x02000000;
        const ACCENT          = 0x04000000;
        const PARENT          = 0x08000000;
        const CHILD           = 0x10000000;
        const ARPEGGIO        = 0x20000000;
        const CHORD_PANEL     = 0x80000000;
    }
}

impl Default for NoteMask {
    fn default() -> Self {
        NoteMask::empty()
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChordMask: u32 {
        const ARPEGGIO = 0b01;
        const NOP      = 0b10;
    }
}

impl Default for ChordMask {
    fn default() -> Self {
        ChordMask::empty()
    }
}
