use super::masks::BeatMask;
use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct Beat {
    pub time: f32,
    pub measure: i16,
    pub beat: i16,
    pub phrase_iteration: i32,
    pub mask: BeatMask,
}

impl SngRead for Beat {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(Beat {
            time: read_f32(r)?,
            measure: read_i16(r)?,
            beat: read_i16(r)?,
            phrase_iteration: read_i32(r)?,
            mask: BeatMask::from_bits_truncate(read_i32(r)?),
        })
    }
}

impl SngWrite for Beat {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        write_i16(w, self.measure)?;
        write_i16(w, self.beat)?;
        write_i32(w, self.phrase_iteration)?;
        write_i32(w, self.mask.bits())?;
        Ok(())
    }
}
