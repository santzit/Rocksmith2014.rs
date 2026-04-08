use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct Phrase {
    pub solo: i8,
    pub disparity: i8,
    pub ignore: i8,
    pub max_difficulty: i32,
    pub iteration_count: i32,
    pub name: [u8; 32],
}

impl SngRead for Phrase {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let solo = read_i8(r)?;
        let disparity = read_i8(r)?;
        let ignore = read_i8(r)?;
        let _ = read_u8(r)?; // padding
        let max_difficulty = read_i32(r)?;
        let iteration_count = read_i32(r)?;
        let mut name = [0u8; 32];
        r.read_exact(&mut name)?;
        Ok(Phrase {
            solo,
            disparity,
            ignore,
            max_difficulty,
            iteration_count,
            name,
        })
    }
}

impl SngWrite for Phrase {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i8(w, self.solo)?;
        write_i8(w, self.disparity)?;
        write_i8(w, self.ignore)?;
        write_u8(w, 0)?; // padding
        write_i32(w, self.max_difficulty)?;
        write_i32(w, self.iteration_count)?;
        w.write_all(&self.name)?;
        Ok(())
    }
}
