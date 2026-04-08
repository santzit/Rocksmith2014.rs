use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct NewLinkedDifficulty {
    pub level_break: i32,
    pub nld_phrases: Vec<i32>,
}

impl SngRead for NewLinkedDifficulty {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let level_break = read_i32(r)?;
        let nld_phrases = read_vec_i32(r)?;
        Ok(NewLinkedDifficulty { level_break, nld_phrases })
    }
}

impl SngWrite for NewLinkedDifficulty {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i32(w, self.level_break)?;
        write_vec_i32(w, &self.nld_phrases)?;
        Ok(())
    }
}
