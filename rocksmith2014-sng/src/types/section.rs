use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone)]
pub struct Section {
    pub name: [u8; 32],
    pub number: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub start_phrase_iteration_id: i32,
    pub end_phrase_iteration_id: i32,
    pub string_mask: [i8; 36],
}

impl Default for Section {
    fn default() -> Self {
        Section {
            name: [0u8; 32],
            number: 0,
            start_time: 0.0,
            end_time: 0.0,
            start_phrase_iteration_id: 0,
            end_phrase_iteration_id: 0,
            string_mask: [0i8; 36],
        }
    }
}

impl SngRead for Section {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mut name = [0u8; 32];
        r.read_exact(&mut name)?;
        let number = read_i32(r)?;
        let start_time = read_f32(r)?;
        let end_time = read_f32(r)?;
        let start_phrase_iteration_id = read_i32(r)?;
        let end_phrase_iteration_id = read_i32(r)?;
        let mut string_mask = [0i8; 36];
        for v in string_mask.iter_mut() {
            *v = read_i8(r)?;
        }
        Ok(Section { name, number, start_time, end_time, start_phrase_iteration_id, end_phrase_iteration_id, string_mask })
    }
}

impl SngWrite for Section {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        w.write_all(&self.name)?;
        write_i32(w, self.number)?;
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_i32(w, self.start_phrase_iteration_id)?;
        write_i32(w, self.end_phrase_iteration_id)?;
        for &v in &self.string_mask {
            write_i8(w, v)?;
        }
        Ok(())
    }
}
