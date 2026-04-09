use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct Tone {
    pub time: f32,
    pub tone_id: i32,
}

impl SngRead for Tone {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(Tone {
            time: read_f32(r)?,
            tone_id: read_i32(r)?,
        })
    }
}

impl SngWrite for Tone {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.tone_id)?;
        Ok(())
    }
}
