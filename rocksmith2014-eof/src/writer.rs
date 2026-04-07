use std::fs::File;
use std::io::{self, Write};

use crate::types::EofProTracks;

fn write_str(out: &mut dyn Write, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    out.write_all(&(bytes.len() as i16).to_le_bytes())?;
    out.write_all(bytes)
}

/// Writes an EOF project file to `output_path`.
pub fn write_eof_project(
    eof_project: &EofProTracks,
    ogg_file: &str,
    output_path: &str,
) -> io::Result<()> {
    let inst = eof_project.get_any_instrumental().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "at least one instrumental arrangement is required",
        )
    })?;

    let mut out = File::create(output_path)?;

    // ---- Header ----
    out.write_all(b"EOFSONH\x00")?; // magic
    out.write_all(&0i64.to_le_bytes())?; // padding
    out.write_all(&1i32.to_le_bytes())?; // revision
    out.write_all(&[0u8])?; // timing format: 0 = ms
    out.write_all(&480i32.to_le_bytes())?; // time division

    // ---- INI strings ----
    let meta = &inst.data.meta;
    let mut ini: Vec<(u8, String)> = Vec::new();
    if !meta.artist_name.is_empty() {
        ini.push((2, meta.artist_name.clone()));
    }
    if !meta.song_name.is_empty() {
        ini.push((3, meta.song_name.clone()));
    }
    if !meta.album_name.is_empty() {
        ini.push((8, meta.album_name.clone()));
    }
    if meta.album_year > 0 {
        ini.push((6, meta.album_year.to_string()));
    }

    out.write_all(&(ini.len() as u16).to_le_bytes())?;
    for (kind, val) in &ini {
        out.write_all(&[*kind])?;
        write_str(&mut out, val)?;
    }

    // ---- INI booleans: 1 entry – "accurate TS" enabled ----
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&(11u8 | 0x80u8).to_le_bytes())?;

    // ---- INI numbers: 1 entry – band difficulty = 255 ----
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&[2u8])?;
    out.write_all(&255u32.to_le_bytes())?;

    // ---- OGG profiles: 1 profile ----
    out.write_all(&1u16.to_le_bytes())?;
    write_str(&mut out, ogg_file)?;
    out.write_all(&0u16.to_le_bytes())?; // orig filename length
    out.write_all(&0u16.to_le_bytes())?; // ogg profile length
    out.write_all(&meta.start_beat.to_le_bytes())?; // MIDI delay
    out.write_all(&0i32.to_le_bytes())?; // profile flags

    // ---- Beats ----
    let n_beats = inst.data.ebeats.len() as i32;
    out.write_all(&n_beats.to_le_bytes())?;
    for beat in &inst.data.ebeats {
        out.write_all(&(beat.time as u32).to_le_bytes())?;
        out.write_all(&120_000u32.to_le_bytes())?; // tempo placeholder
        out.write_all(&0u32.to_le_bytes())?; // flags
    }

    // ---- Events: 0 ----
    out.write_all(&0i32.to_le_bytes())?;

    // ---- Custom data: 0 ----
    out.write_all(&0u32.to_le_bytes())?;

    // ---- Tracks: 0 ----
    out.write_all(&0i32.to_le_bytes())?;

    Ok(())
}
