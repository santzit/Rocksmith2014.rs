use rocksmith2014_xml::InstrumentalArrangement;

/// An imported instrumental arrangement.
#[derive(Debug, Clone)]
pub struct ImportedArrangement {
    pub data: InstrumentalArrangement,
    pub custom_name: String,
}

/// Imported vocals (placeholder).
#[derive(Debug, Clone)]
pub struct ImportedVocals {
    pub custom_name: String,
}

/// Collection of EOF pro-guitar tracks.
#[derive(Debug, Clone, Default)]
pub struct EofProTracks {
    pub part_guitar: Vec<ImportedArrangement>,
    pub part_bass: Vec<ImportedArrangement>,
    pub part_bonus: Option<ImportedArrangement>,
    pub part_vocals: Option<ImportedVocals>,
}

impl EofProTracks {
    pub fn get_any_instrumental(&self) -> Option<&ImportedArrangement> {
        self.part_guitar
            .first()
            .or_else(|| self.part_bass.first())
            .or(self.part_bonus.as_ref())
    }
}

/// EOF time signature variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EofTimeSignature {
    Ts2_4,
    Ts3_4,
    Ts4_4,
    Ts5_4,
    Ts6_4,
    Custom(u32, u32),
}
