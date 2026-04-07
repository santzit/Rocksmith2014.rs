use std::fs::File;
use std::io::{self, BufWriter, Write};
use rocksmith2014_xml::InstrumentalArrangement;
use crate::types::{EofEvent, IniString, IniStringType, TimeSignature};
use crate::write_utils::*;
use crate::helpers::{get_time_signatures, infer_time_signatures};
use crate::event_converter::{create_eof_events, unify_events};
use crate::beat_writer::write_beats;
use crate::ini_writers::{write_ini_strings, write_ini_booleans, write_ini_numbers};
use crate::vocals_writer::write_vocals_track;
use crate::pro_guitar_writer::{write_empty_pro_guitar_track, write_pro_track};

#[derive(Debug, Clone)]
pub struct ImportedArrangement {
    pub data: InstrumentalArrangement,
    pub custom_name: String,
}

#[derive(Debug, Clone)]
pub struct Vocal {
    pub time: i32,
    pub note: u8,
    pub length: i32,
    pub lyric: String,
}

#[derive(Debug, Clone)]
pub struct ImportedVocals {
    pub vocals: Vec<Vocal>,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EofProTracks {
    pub part_guitar: Vec<ImportedArrangement>,
    pub part_bass: Vec<ImportedArrangement>,
    pub part_bonus: Option<ImportedArrangement>,
    pub part_vocals: Option<ImportedVocals>,
}

impl EofProTracks {
    pub fn get_any_instrumental(&self) -> &ImportedArrangement {
        self.part_guitar.first()
            .or_else(|| self.part_bass.first())
            .or_else(|| self.part_bonus.as_ref())
            .expect("One instrumental arrangement needed for EOF export.")
    }

    pub fn all_instrumentals(&self) -> Vec<&ImportedArrangement> {
        let mut result: Vec<&ImportedArrangement> = Vec::new();
        result.extend(self.part_guitar.iter());
        result.extend(self.part_bass.iter());
        if let Some(bonus) = &self.part_bonus {
            result.push(bonus);
        }
        result
    }
}

enum ProGuitarTrack<'a> {
    Actual { name: &'a str, imported: &'a ImportedArrangement },
    Empty { name: &'a str },
}

enum Track<'a> {
    Track0,
    Legacy { name: &'a str, behavior: u8, ty: u8, lanes: u8 },
    Vocals,
    ProGuitar(ProGuitarTrack<'a>),
}

fn get_tracks<'a>(project: &'a EofProTracks) -> Vec<Track<'a>> {
    let get_or_empty = |name: &'a str, arr: &'a [ImportedArrangement], index: usize| -> Track<'a> {
        match arr.get(index) {
            Some(imported) => Track::ProGuitar(ProGuitarTrack::Actual { name, imported }),
            None => Track::ProGuitar(ProGuitarTrack::Empty { name }),
        }
    };

    let mut tracks = vec![
        Track::Track0,
        Track::Legacy { name: "PART GUITAR", behavior: 1, ty: 1, lanes: 5 },
        Track::Legacy { name: "PART BASS", behavior: 1, ty: 2, lanes: 5 },
        Track::Legacy { name: "PART GUITAR COOP", behavior: 1, ty: 3, lanes: 5 },
        Track::Legacy { name: "PART RHYTHM", behavior: 1, ty: 4, lanes: 5 },
        Track::Legacy { name: "PART DRUMS", behavior: 2, ty: 5, lanes: 5 },
        Track::Vocals,
        Track::Legacy { name: "PART KEYS", behavior: 4, ty: 7, lanes: 5 },
        get_or_empty("PART REAL_BASS", &project.part_bass, 0),
        get_or_empty("PART REAL_GUITAR", &project.part_guitar, 0),
        Track::Legacy { name: "PART DANCE", behavior: 7, ty: 10, lanes: 4 },
        get_or_empty("PART REAL_BASS_22", &project.part_bass, 1),
        get_or_empty("PART REAL_GUITAR_22", &project.part_guitar, 1),
        Track::Legacy { name: "PART REAL_DRUMS_PS", behavior: 2, ty: 13, lanes: 5 },
    ];

    if let Some(bonus) = &project.part_bonus {
        tracks.push(Track::ProGuitar(ProGuitarTrack::Actual {
            name: "PART REAL_GUITAR_BONUS",
            imported: bonus,
        }));
    }

    tracks
}

fn get_track_index(tracks: &[Track<'_>], arr: &InstrumentalArrangement) -> usize {
    tracks.iter().position(|t| match t {
        Track::ProGuitar(ProGuitarTrack::Actual { imported, .. }) => {
            std::ptr::eq(&imported.data as *const _, arr as *const _)
        }
        _ => false,
    }).unwrap_or(0)
}

fn write_track(writer: &mut impl Write, track: &Track<'_>, vocals: Option<&ImportedVocals>) -> io::Result<()> {
    match track {
        Track::Track0 => {
            write_eof_string(writer, "")?;
            write_u8(writer, 0)?; // format
            write_u8(writer, 0)?; // behaviour
            write_u8(writer, 0)?; // type
            write_i8(writer, 0)?; // difficulty
            write_i32_le(writer, 0)?; // flags
            write_u16_le(writer, 65535)?; // compliance flags
            write_u16_le(writer, 0)?; // sections
            write_i32_le(writer, 1)?; // custom data blocks
            write_i32_le(writer, 4)?; // block size
            write_u32_le(writer, 0xFFFFFFFF)?;
        }
        Track::Legacy { name, behavior, ty, lanes } => {
            write_eof_string(writer, name)?;
            write_u8(writer, 1)?; // format
            write_u8(writer, *behavior)?;
            write_u8(writer, *ty)?;
            write_i8(writer, -1)?;
            let flags: u32 = if *name == "PART DRUMS" { 4278190080u32 } else { 0 };
            write_u32_le(writer, flags)?;
            write_u16_le(writer, 0)?; // compliance flags
            write_u8(writer, *lanes)?;
            write_u32_le(writer, 0)?; // notes
            write_u16_le(writer, 0)?; // section types
            write_i32_le(writer, 1)?; // custom data blocks
            write_i32_le(writer, 4)?; // block size
            write_u32_le(writer, 0xFFFFFFFF)?;
        }
        Track::Vocals => {
            write_vocals_track(writer, vocals)?;
        }
        Track::ProGuitar(ProGuitarTrack::Actual { name, imported }) => {
            write_pro_track(writer, name, imported)?;
        }
        Track::ProGuitar(ProGuitarTrack::Empty { name }) => {
            write_empty_pro_guitar_track(writer, name)?;
        }
    }
    Ok(())
}

fn write_header(writer: &mut impl Write) -> io::Result<()> {
    writer.write_all(b"EOFSONH\x00")?;
    write_i64_le(writer, 0)?;
    write_i32_le(writer, 1)?; // revision
    write_u8(writer, 0)?; // timing format
    write_i32_le(writer, 480)?; // time division
    Ok(())
}

fn write_ogg_profiles(writer: &mut impl Write, ogg_file: &str, delay: i32) -> io::Result<()> {
    write_u16_le(writer, 1)?; // count
    write_eof_string(writer, ogg_file)?;
    write_u16_le(writer, 0)?; // orig len
    write_u16_le(writer, 0)?; // profile len
    write_i32_le(writer, delay)?;
    write_i32_le(writer, 0)?; // flags
    Ok(())
}

fn write_events(writer: &mut impl Write, events: &[EofEvent]) -> io::Result<()> {
    write_i32_le(writer, events.len() as i32)?;
    for e in events {
        write_eof_string(writer, &e.text)?;
        write_i32_le(writer, e.beat_number)?;
        write_u16_le(writer, e.track_number)?;
        write_u16_le(writer, e.flag.bits())?;
    }
    Ok(())
}

pub fn write_eof_project(ogg_file: &str, path: &str, project: &EofProTracks) -> io::Result<()> {
    let inst = &project.get_any_instrumental().data;

    let tracks = get_tracks(project);

    let instrumentals = project.all_instrumentals();
    let beats = &inst.ebeats;

    let mut events: Vec<EofEvent> = instrumentals.iter()
        .flat_map(|imported| {
            create_eof_events(
                &|arr: &InstrumentalArrangement| get_track_index(&tracks, arr),
                beats,
                &imported.data,
            )
        })
        .collect();
    events.sort_by_key(|e| e.beat_number);
    let events = unify_events(instrumentals.len(), events);

    let time_signatures: Vec<(i32, TimeSignature)> = if inst.events.iter().any(|e| e.code.starts_with("TS")) {
        get_time_signatures(&inst.events)
    } else {
        infer_time_signatures(beats)
    };

    let ini_strings: Vec<IniString> = {
        let mut s = Vec::new();
        if !inst.meta.artist_name.is_empty() {
            s.push(IniString { string_type: IniStringType::Artist, value: inst.meta.artist_name.clone() });
        }
        if !inst.meta.song_name.is_empty() {
            s.push(IniString { string_type: IniStringType::Title, value: inst.meta.song_name.clone() });
        }
        if !inst.meta.album_name.is_empty() {
            s.push(IniString { string_type: IniStringType::Album, value: inst.meta.album_name.clone() });
        }
        if inst.meta.album_year > 0 {
            s.push(IniString { string_type: IniStringType::Year, value: inst.meta.album_year.to_string() });
        }
        s
    };

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write_header(&mut writer)?;
    write_ini_strings(&mut writer, &ini_strings)?;
    write_ini_booleans(&mut writer)?;
    write_ini_numbers(&mut writer)?;
    write_ogg_profiles(&mut writer, ogg_file, inst.meta.start_beat)?;
    write_beats(&mut writer, inst, &events, &time_signatures)?;
    write_events(&mut writer, &events)?;
    write_u32_le(&mut writer, 0)?; // custom data count

    write_i32_le(&mut writer, tracks.len() as i32)?;
    for track in &tracks {
        write_track(&mut writer, track, project.part_vocals.as_ref())?;
    }

    writer.flush()?;
    Ok(())
}
