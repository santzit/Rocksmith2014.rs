/// Tuning as string offsets for each string.
///
/// Mirrors `Tuning.fs` from `Rocksmith2014.Common`.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tuning {
    pub string0: i16,
    pub string1: i16,
    pub string2: i16,
    pub string3: i16,
    pub string4: i16,
    pub string5: i16,
}

impl Tuning {
    pub fn to_array(&self) -> [i16; 6] {
        [
            self.string0,
            self.string1,
            self.string2,
            self.string3,
            self.string4,
            self.string5,
        ]
    }

    pub fn from_array(strings: [i16; 6]) -> Self {
        Self {
            string0: strings[0],
            string1: strings[1],
            string2: strings[2],
            string3: strings[3],
            string4: strings[4],
            string5: strings[5],
        }
    }
}
