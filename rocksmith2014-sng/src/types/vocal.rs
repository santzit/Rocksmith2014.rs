use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Vocal {
    pub time: f32,
    pub note: i32,
    pub length: f32,
    pub lyric: [u8; 48],
}

impl Default for Vocal {
    fn default() -> Self {
        Vocal {
            time: 0.0,
            note: 0,
            length: 0.0,
            lyric: [0u8; 48],
        }
    }
}

impl SngRead for Vocal {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let time = read_f32(r)?;
        let note = read_i32(r)?;
        let length = read_f32(r)?;
        let mut lyric = [0u8; 48];
        r.read_exact(&mut lyric)?;
        Ok(Vocal {
            time,
            note,
            length,
            lyric,
        })
    }
}

impl SngWrite for Vocal {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.note)?;
        write_f32(w, self.length)?;
        w.write_all(&self.lyric)?;
        Ok(())
    }
}
