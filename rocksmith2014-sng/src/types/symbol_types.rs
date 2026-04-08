use std::io::{Read, Write};
use crate::binary_helpers::*;

#[derive(Debug, Clone, Default)]
pub struct SymbolsHeader {
    pub id: i32,
    pub unk2: i32,
    pub unk3: i32,
    pub unk4: i32,
    pub unk5: i32,
    pub unk6: i32,
    pub unk7: i32,
    pub unk8: i32,
}

impl SngRead for SymbolsHeader {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(SymbolsHeader {
            id: read_i32(r)?,
            unk2: read_i32(r)?,
            unk3: read_i32(r)?,
            unk4: read_i32(r)?,
            unk5: read_i32(r)?,
            unk6: read_i32(r)?,
            unk7: read_i32(r)?,
            unk8: read_i32(r)?,
        })
    }
}

impl SngWrite for SymbolsHeader {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_i32(w, self.id)?;
        write_i32(w, self.unk2)?;
        write_i32(w, self.unk3)?;
        write_i32(w, self.unk4)?;
        write_i32(w, self.unk5)?;
        write_i32(w, self.unk6)?;
        write_i32(w, self.unk7)?;
        write_i32(w, self.unk8)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SymbolsTexture {
    pub font: [u8; 128],
    pub font_path_length: i32,
    pub width: i32,
    pub height: i32,
}

impl Default for SymbolsTexture {
    fn default() -> Self {
        SymbolsTexture { font: [0u8; 128], font_path_length: 0, width: 0, height: 0 }
    }
}

impl SngRead for SymbolsTexture {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mut font = [0u8; 128];
        r.read_exact(&mut font)?;
        let font_path_length = read_i32(r)?;
        let _ = read_i32(r)?; // unknown
        let width = read_i32(r)?;
        let height = read_i32(r)?;
        Ok(SymbolsTexture { font, font_path_length, width, height })
    }
}

impl SngWrite for SymbolsTexture {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        w.write_all(&self.font)?;
        write_i32(w, self.font_path_length)?;
        write_i32(w, 0)?; // unknown
        write_i32(w, self.width)?;
        write_i32(w, self.height)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rect {
    pub ymin: f32,
    pub xmin: f32,
    pub ymax: f32,
    pub xmax: f32,
}

impl SngRead for Rect {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        Ok(Rect { ymin: read_f32(r)?, xmin: read_f32(r)?, ymax: read_f32(r)?, xmax: read_f32(r)? })
    }
}

impl SngWrite for Rect {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        write_f32(w, self.ymin)?;
        write_f32(w, self.xmin)?;
        write_f32(w, self.ymax)?;
        write_f32(w, self.xmax)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolDefinition {
    pub symbol: [u8; 12],
    pub outer: Rect,
    pub inner: Rect,
}

impl SngRead for SymbolDefinition {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mut symbol = [0u8; 12];
        r.read_exact(&mut symbol)?;
        let outer = Rect::sng_read(r)?;
        let inner = Rect::sng_read(r)?;
        Ok(SymbolDefinition { symbol, outer, inner })
    }
}

impl SngWrite for SymbolDefinition {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        w.write_all(&self.symbol)?;
        self.outer.sng_write(w)?;
        self.inner.sng_write(w)?;
        Ok(())
    }
}
