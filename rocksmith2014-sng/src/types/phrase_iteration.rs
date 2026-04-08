use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct PhraseIteration {
    pub phrase_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub difficulty: [i32; 3],
}

impl SngRead for PhraseIteration {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let phrase_id = read_i32(r)?;
        let start_time = read_f32(r)?;
        let end_time = read_f32(r)?;
        let difficulty = [read_i32(r)?, read_i32(r)?, read_i32(r)?];
        Ok(PhraseIteration { phrase_id, start_time, end_time, difficulty })
    }
}

impl SngWrite for PhraseIteration {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i32(w, self.phrase_id)?;
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_i32(w, self.difficulty[0])?;
        write_i32(w, self.difficulty[1])?;
        write_i32(w, self.difficulty[2])?;
        Ok(())
    }
}
