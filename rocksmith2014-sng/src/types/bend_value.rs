use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct BendValue {
    pub time: f32,
    pub step: f32,
    pub unused: u32,
}

impl SngRead for BendValue {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(BendValue {
            time: read_f32(r)?,
            step: read_f32(r)?,
            unused: read_u32(r)?,
        })
    }
}

impl SngWrite for BendValue {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        write_f32(w, self.step)?;
        write_u32(w, self.unused)?;
        Ok(())
    }
}
