use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct PhraseExtraInfo {
    pub phrase_id: i32,
    pub difficulty: i32,
    pub empty: i32,
    pub level_jump: i8,
    pub redundant: i16,
}

impl SngRead for PhraseExtraInfo {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let phrase_id = read_i32(r)?;
        let difficulty = read_i32(r)?;
        let empty = read_i32(r)?;
        let level_jump = read_i8(r)?;
        let redundant = read_i16(r)?;
        let _ = read_u8(r)?; // padding
        Ok(PhraseExtraInfo { phrase_id, difficulty, empty, level_jump, redundant })
    }
}

impl SngWrite for PhraseExtraInfo {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i32(w, self.phrase_id)?;
        write_i32(w, self.difficulty)?;
        write_i32(w, self.empty)?;
        write_i8(w, self.level_jump)?;
        write_i16(w, self.redundant)?;
        write_u8(w, 0)?; // padding
        Ok(())
    }
}
