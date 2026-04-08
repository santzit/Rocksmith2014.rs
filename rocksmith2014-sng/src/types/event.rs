use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f32,
    pub name: [u8; 256],
}

impl Default for Event {
    fn default() -> Self {
        Event {
            time: 0.0,
            name: [0u8; 256],
        }
    }
}

impl SngRead for Event {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let time = read_f32(r)?;
        let mut name = [0u8; 256];
        r.read_exact(&mut name)?;
        Ok(Event { time, name })
    }
}

impl SngWrite for Event {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        w.write_all(&self.name)?;
        Ok(())
    }
}
