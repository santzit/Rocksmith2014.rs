use std::io::{Read, Write};
use crate::binary_helpers::*;
use super::bend_data32::BendData32;

#[derive(Debug, Clone)]
pub struct ChordNotes {
    pub mask: [u32; 6],
    pub bend_data: [BendData32; 6],
    pub slide_to: [i8; 6],
    pub slide_unpitch_to: [i8; 6],
    pub vibrato: [i16; 6],
}

impl Default for ChordNotes {
    fn default() -> Self {
        ChordNotes {
            mask: [0u32; 6],
            bend_data: std::array::from_fn(|_| BendData32::default()),
            slide_to: [0i8; 6],
            slide_unpitch_to: [0i8; 6],
            vibrato: [0i16; 6],
        }
    }
}

impl SngRead for ChordNotes {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mut mask = [0u32; 6];
        for v in mask.iter_mut() { *v = read_u32(r)?; }
        let mut bend_data: [BendData32; 6] = std::array::from_fn(|_| BendData32::default());
        for v in bend_data.iter_mut() { *v = BendData32::sng_read(r)?; }
        let mut slide_to = [0i8; 6];
        for v in slide_to.iter_mut() { *v = read_i8(r)?; }
        let mut slide_unpitch_to = [0i8; 6];
        for v in slide_unpitch_to.iter_mut() { *v = read_i8(r)?; }
        let mut vibrato = [0i16; 6];
        for v in vibrato.iter_mut() { *v = read_i16(r)?; }
        Ok(ChordNotes { mask, bend_data, slide_to, slide_unpitch_to, vibrato })
    }
}

impl SngWrite for ChordNotes {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        for &v in &self.mask { write_u32(w, v)?; }
        for v in &self.bend_data { v.sng_write(w)?; }
        for &v in &self.slide_to { write_i8(w, v)?; }
        for &v in &self.slide_unpitch_to { write_i8(w, v)?; }
        for &v in &self.vibrato { write_i16(w, v)?; }
        Ok(())
    }
}
