use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct DNA {
    pub time: f32,
    pub dna_id: i32,
}

impl SngRead for DNA {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(DNA { time: read_f32(r)?, dna_id: read_i32(r)? })
    }
}

impl SngWrite for DNA {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.dna_id)?;
        Ok(())
    }
}
