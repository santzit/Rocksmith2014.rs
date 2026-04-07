use crate::eof_project_writer::{ImportedVocals, Vocal};
use crate::types::EofSection;
use crate::write_utils::*;
use std::io::{self, Write};

fn write_vocal(writer: &mut impl Write, vocal: &Vocal) -> io::Result<()> {
    let lyric = if vocal.lyric.ends_with('+') {
        &vocal.lyric[..vocal.lyric.len() - 1]
    } else {
        &vocal.lyric
    };
    write_eof_string(writer, lyric)?;
    write_u8(writer, 0)?; // lyric set number
    let note = if vocal.note == 254 { 0 } else { vocal.note };
    write_u8(writer, note)?;
    write_i32_le(writer, vocal.time)?;
    write_i32_le(writer, vocal.length)?;
    write_u32_le(writer, 0)?; // flags
    Ok(())
}

fn get_section_times(vocals: &[Vocal]) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let mut start_time: Option<i32> = None;

    for v in vocals {
        let s = start_time.get_or_insert(v.time);
        if v.lyric.ends_with('+') {
            result.push((*s, v.time + v.length));
            start_time = None;
        }
    }
    result
}

pub fn write_vocals_track(
    writer: &mut impl Write,
    vocals_data: Option<&ImportedVocals>,
) -> io::Result<()> {
    let vocals: Vec<&Vocal> = vocals_data
        .map(|x| x.vocals.iter().collect())
        .unwrap_or_default();

    let (track_flags, custom_name) = match vocals_data.and_then(|x| x.custom_name.as_deref()) {
        Some(name) if !name.is_empty() => (4278190082u32, name),
        _ => (4278190080u32, ""),
    };

    let sections: Vec<EofSection> =
        get_section_times(&vocals.iter().map(|v| (*v).clone()).collect::<Vec<_>>())
            .into_iter()
            .map(|(start, end)| EofSection::create(0, start as u32, end as u32, 0))
            .collect();

    // Header
    write_eof_string(writer, "PART VOCALS")?;
    write_u8(writer, 2)?; // format
    write_u8(writer, 3)?; // behaviour
    write_u8(writer, 6)?; // type
    write_i8(writer, -1)?; // difficulty
    write_u32_le(writer, track_flags)?;
    write_u16_le(writer, 0)?; // compliance flags

    if !custom_name.is_empty() {
        write_eof_string(writer, custom_name)?;
    }

    write_i8(writer, 5)?; // MIDI tone

    // Vocals
    write_i32_le(writer, vocals.len() as i32)?;
    for v in &vocals {
        write_vocal(writer, v)?;
    }

    // Section types: 1 type (lyrics)
    write_u16_le(writer, 1)?;
    write_u16_le(writer, 5)?; // type 5 = lyrics
    write_sections(writer, &sections)?;

    // Custom data blocks
    write_i32_le(writer, 1)?;
    write_i32_le(writer, 4)?; // block size
    write_u32_le(writer, 0xFFFFFFFF)?;

    Ok(())
}
