#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelInput {
    pub key: Option<String>,
    pub arr_name: Option<String>,
    pub path: f32,
    pub phrase: Option<String>,
    pub levels: f32,
    pub unique_levels: f32,
    pub length_ms: f32,
    pub length_beats: f32,
    pub tempo: f32,
    pub notes: f32,
    pub repeated_notes: f32,
    pub chords: f32,
    pub repeated_chords: f32,
    pub tech_count: f32,
    pub palm_mutes: f32,
    pub bends: f32,
    pub harmonics: f32,
    pub pharmonics: f32,
    pub taps: f32,
    pub tremolos: f32,
    pub vibratos: f32,
    pub slides: f32,
    pub unp_slides: f32,
    pub ignored: f32,
    pub anchors: f32,
    pub max_chord_strings: f32,
    pub solo: Option<String>,
}

impl ModelInput {
    pub fn to_onnx_features(&self) -> [f32; 20] {
        [
            self.path,
            self.length_ms,
            self.length_beats,
            self.tempo,
            self.notes,
            self.repeated_notes,
            self.chords,
            self.tech_count,
            self.palm_mutes,
            self.bends,
            self.harmonics,
            self.pharmonics,
            self.taps,
            self.tremolos,
            self.vibratos,
            self.slides,
            self.unp_slides,
            self.anchors,
            self.max_chord_strings,
            self.solo_feature(),
        ]
    }

    fn solo_feature(&self) -> f32 {
        self.solo
            .as_deref()
            .map(str::trim)
            .map(|s| {
                if s.eq_ignore_ascii_case("true") || s == "1" {
                    1.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ModelInput;

    #[test]
    fn onnx_feature_vector_matches_dd_level_counter_order() {
        let input = ModelInput {
            key: None,
            arr_name: None,
            path: 1.0,
            phrase: None,
            levels: 0.0,
            unique_levels: 0.0,
            length_ms: 2.0,
            length_beats: 3.0,
            tempo: 4.0,
            notes: 5.0,
            repeated_notes: 6.0,
            chords: 7.0,
            repeated_chords: 0.0,
            tech_count: 8.0,
            palm_mutes: 9.0,
            bends: 10.0,
            harmonics: 11.0,
            pharmonics: 12.0,
            taps: 13.0,
            tremolos: 14.0,
            vibratos: 15.0,
            slides: 16.0,
            unp_slides: 17.0,
            ignored: 0.0,
            anchors: 18.0,
            max_chord_strings: 19.0,
            solo: Some("1".to_string()),
        };

        assert_eq!(
            input.to_onnx_features(),
            [
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 1.0,
            ]
        );
    }
}
