use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct FingerPrint {
    pub chord_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub first_note_time: f32,
    pub last_note_time: f32,
}

impl SngRead for FingerPrint {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(FingerPrint {
            chord_id: read_i32(r)?,
            start_time: read_f32(r)?,
            end_time: read_f32(r)?,
            first_note_time: read_f32(r)?,
            last_note_time: read_f32(r)?,
        })
    }
}

impl SngWrite for FingerPrint {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i32(w, self.chord_id)?;
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_f32(w, self.first_note_time)?;
        write_f32(w, self.last_note_time)?;
        Ok(())
    }
}
