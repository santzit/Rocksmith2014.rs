use std::io::{self, Write};

pub fn write_i8(w: &mut impl Write, v: i8) -> io::Result<()> {
    w.write_all(&[v as u8])
}

pub fn write_u8(w: &mut impl Write, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

pub fn write_i16_le(w: &mut impl Write, v: i16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u16_le(w: &mut impl Write, v: u16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_i32_le(w: &mut impl Write, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u32_le(w: &mut impl Write, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_i64_le(w: &mut impl Write, v: i64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

/// Writes: i16 length (as bytes) + ASCII bytes
pub fn write_eof_string(w: &mut impl Write, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    write_i16_le(w, bytes.len() as i16)?;
    w.write_all(bytes)
}

pub fn write_notes(w: &mut impl Write, notes: &[crate::types::EofNote]) -> io::Result<()> {
    write_i32_le(w, notes.len() as i32)?;
    for note in notes {
        write_eof_string(w, &note.chord_name)?;
        write_u8(w, note.chord_number)?;
        write_u8(w, note.difficulty)?;
        write_u8(w, note.bit_flag)?;
        write_u8(w, note.ghost_bit_flag)?;
        w.write_all(&note.frets)?;
        write_u8(w, note.legacy_bit_flags)?;
        write_u32_le(w, note.position)?;
        write_u32_le(w, note.length)?;
        write_u32_le(w, note.flags.bits())?;
        if let Some(f) = note.slide_end_fret {
            write_u8(w, f)?;
        }
        if let Some(b) = note.bend_strength {
            write_u8(w, b)?;
        }
        if let Some(u) = note.unpitched_slide_end_fret {
            write_u8(w, u)?;
        }
        if !note.extended_note_flags.is_empty() {
            write_u32_le(w, note.extended_note_flags.bits())?;
        }
    }
    Ok(())
}

pub fn write_sections(w: &mut impl Write, sections: &[crate::types::EofSection]) -> io::Result<()> {
    write_i32_le(w, sections.len() as i32)?;
    for s in sections {
        write_eof_string(w, &s.name)?;
        write_u8(w, s.ty)?;
        write_u32_le(w, s.start_time)?;
        write_u32_le(w, s.end_time)?;
        write_u32_le(w, s.flags)?;
    }
    Ok(())
}
