use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct AnchorExtension {
    pub beat_time: f32,
    pub fret_id: i8,
}

impl SngRead for AnchorExtension {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let beat_time = read_f32(r)?;
        let fret_id = read_i8(r)?;
        let mut pad = [0u8; 7];
        r.read_exact(&mut pad)?;
        Ok(AnchorExtension { beat_time, fret_id })
    }
}

impl SngWrite for AnchorExtension {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.beat_time)?;
        write_i8(w, self.fret_id)?;
        w.write_all(&[0u8; 7])?;
        Ok(())
    }
}
