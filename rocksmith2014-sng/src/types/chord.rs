use super::masks::ChordMask;
use crate::binary_helpers::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct Chord {
    pub mask: ChordMask,
    pub frets: [i8; 6],
    pub fingers: [i8; 6],
    pub notes: [i32; 6],
    pub name: [u8; 32],
}

impl SngRead for Chord {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mask = ChordMask::from_bits_truncate(read_u32(r)?);
        let mut frets = [0i8; 6];
        for v in frets.iter_mut() {
            *v = read_i8(r)?;
        }
        let mut fingers = [0i8; 6];
        for v in fingers.iter_mut() {
            *v = read_i8(r)?;
        }
        let mut notes = [0i32; 6];
        for v in notes.iter_mut() {
            *v = read_i32(r)?;
        }
        let mut name = [0u8; 32];
        r.read_exact(&mut name)?;
        Ok(Chord {
            mask,
            frets,
            fingers,
            notes,
            name,
        })
    }
}

impl SngWrite for Chord {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_u32(w, self.mask.bits())?;
        for &v in &self.frets {
            write_i8(w, v)?;
        }
        for &v in &self.fingers {
            write_i8(w, v)?;
        }
        for &v in &self.notes {
            write_i32(w, v)?;
        }
        w.write_all(&self.name)?;
        Ok(())
    }
}
