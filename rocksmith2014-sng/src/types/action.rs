use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone)]
pub struct Action {
    pub time: f32,
    pub action_name: [u8; 256],
}

impl Default for Action {
    fn default() -> Self {
        Action { time: 0.0, action_name: [0u8; 256] }
    }
}

impl SngRead for Action {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let time = read_f32(r)?;
        let mut action_name = [0u8; 256];
        r.read_exact(&mut action_name)?;
        Ok(Action { time, action_name })
    }
}

impl SngWrite for Action {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.time)?;
        w.write_all(&self.action_name)?;
        Ok(())
    }
}
