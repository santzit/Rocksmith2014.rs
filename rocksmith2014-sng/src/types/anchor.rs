use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct Anchor {
    pub start_time: f32,
    pub end_time: f32,
    pub first_note_time: f32,
    pub last_note_time: f32,
    pub fret_id: i8,
    pub width: i32,
    pub phrase_iteration_id: i32,
}

impl SngRead for Anchor {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let start_time = read_f32(r)?;
        let end_time = read_f32(r)?;
        let first_note_time = read_f32(r)?;
        let last_note_time = read_f32(r)?;
        let fret_id = read_i8(r)?;
        let mut pad = [0u8; 3];
        r.read_exact(&mut pad)?;
        let width = read_i32(r)?;
        let phrase_iteration_id = read_i32(r)?;
        Ok(Anchor { start_time, end_time, first_note_time, last_note_time, fret_id, width, phrase_iteration_id })
    }
}

impl SngWrite for Anchor {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_f32(w, self.first_note_time)?;
        write_f32(w, self.last_note_time)?;
        write_i8(w, self.fret_id)?;
        w.write_all(&[0u8; 3])?;
        write_i32(w, self.width)?;
        write_i32(w, self.phrase_iteration_id)?;
        Ok(())
    }
}
