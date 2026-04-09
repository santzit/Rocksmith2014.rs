/// Arrangement properties that describe techniques and characteristics.
///
/// Mirrors `ArrangementProperties.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrangementProperties {
    pub represent: u8,
    #[serde(rename = "bonusArr")]
    pub bonus_arr: u8,
    #[serde(rename = "standardTuning")]
    pub standard_tuning: u8,
    #[serde(rename = "nonStandardChords")]
    pub non_standard_chords: u8,
    #[serde(rename = "barreChords")]
    pub barre_chords: u8,
    #[serde(rename = "powerChords")]
    pub power_chords: u8,
    #[serde(rename = "dropDPower")]
    pub drop_d_power: u8,
    #[serde(rename = "openChords")]
    pub open_chords: u8,
    #[serde(rename = "fingerPicking")]
    pub finger_picking: u8,
    #[serde(rename = "pickDirection")]
    pub pick_direction: u8,
    #[serde(rename = "doubleStops")]
    pub double_stops: u8,
    #[serde(rename = "palmMutes")]
    pub palm_mutes: u8,
    pub harmonics: u8,
    #[serde(rename = "pinchHarmonics")]
    pub pinch_harmonics: u8,
    pub hopo: u8,
    pub tremolo: u8,
    pub slides: u8,
    #[serde(rename = "unpitchedSlides")]
    pub unpitched_slides: u8,
    pub bends: u8,
    pub tapping: u8,
    pub vibrato: u8,
    #[serde(rename = "fretHandMutes")]
    pub fret_hand_mutes: u8,
    #[serde(rename = "slapPop")]
    pub slap_pop: u8,
    #[serde(rename = "twoFingerPicking")]
    pub two_finger_picking: u8,
    #[serde(rename = "fifthsAndOctaves")]
    pub fifths_and_octaves: u8,
    pub syncopation: u8,
    #[serde(rename = "bassPick")]
    pub bass_pick: u8,
    pub sustain: u8,
    #[serde(rename = "pathLead")]
    pub path_lead: u8,
    #[serde(rename = "pathRhythm")]
    pub path_rhythm: u8,
    #[serde(rename = "pathBass")]
    pub path_bass: u8,
    #[serde(rename = "routeMask")]
    pub route_mask: u8,
}
