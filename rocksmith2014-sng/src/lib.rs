use std::io::{self, Read, Write};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use aes::Aes256;
use ctr::{Ctr128BE, cipher::{KeyIvInit, StreamCipher}};
use rand::RngCore;

type AesCtr = Ctr128BE<Aes256>;

const PC_KEY: [u8; 32] = [
    0xCB, 0x64, 0x8D, 0xF3, 0xD1, 0x2A, 0x16, 0xBF, 0x71, 0x70, 0x14, 0x14, 0xE6, 0x96, 0x19, 0xEC,
    0x17, 0x1C, 0xCA, 0x5D, 0x2A, 0x14, 0x2E, 0x3E, 0x59, 0xDE, 0x7A, 0xDD, 0xA1, 0x8A, 0x3A, 0x30,
];
const MAC_KEY: [u8; 32] = [
    0x98, 0x21, 0x33, 0x0E, 0x34, 0xB9, 0x1F, 0x70, 0xD0, 0xA4, 0x8C, 0xBD, 0x62, 0x59, 0x93, 0x12,
    0x69, 0x70, 0xCE, 0xA0, 0x91, 0x92, 0xC0, 0xE6, 0xCD, 0xA6, 0x76, 0xCC, 0x98, 0x38, 0x28, 0x9D,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Pc,
    Mac,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid SNG header")]
    InvalidHeader,
    #[error("Crypto error")]
    Crypto,
}

pub type Result<T> = std::result::Result<T, Error>;

// --- Bitflags ---

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BeatMask: i32 {
        const FIRST_BEAT_OF_MEASURE = 0b01;
        const EVEN_MEASURE          = 0b10;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NoteMask: u32 {
        const CHORD           = 0x00000002;
        const OPEN            = 0x00000004;
        const FRET_HAND_MUTE  = 0x00000008;
        const TREMOLO         = 0x00000010;
        const HARMONIC        = 0x00000020;
        const PALM_MUTE       = 0x00000040;
        const SLAP            = 0x00000080;
        const PLUCK           = 0x00000100;
        const HAMMER_ON       = 0x00000200;
        const PULL_OFF        = 0x00000400;
        const SLIDE           = 0x00000800;
        const BEND            = 0x00001000;
        const SUSTAIN         = 0x00002000;
        const TAP             = 0x00004000;
        const PINCH_HARMONIC  = 0x00008000;
        const VIBRATO         = 0x00010000;
        const MUTE            = 0x00020000;
        const IGNORE          = 0x00040000;
        const LEFT_HAND       = 0x00080000;
        const RIGHT_HAND      = 0x00100000;
        const HIGH_DENSITY    = 0x00200000;
        const UNPITCHED_SLIDE = 0x00400000;
        const SINGLE          = 0x00800000;
        const CHORD_NOTES     = 0x01000000;
        const DOUBLE_STOP     = 0x02000000;
        const ACCENT          = 0x04000000;
        const PARENT          = 0x08000000;
        const CHILD           = 0x10000000;
        const ARPEGGIO        = 0x20000000;
        const CHORD_PANEL     = 0x80000000;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChordMask: u32 {
        const ARPEGGIO = 0b01;
        const NOP      = 0b10;
    }
}

// --- Primitives ---

fn read_u8<R: Read>(r: &mut R) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i8<R: Read>(r: &mut R) -> io::Result<i8> {
    Ok(read_u8(r)? as i8)
}

fn read_i16<R: Read>(r: &mut R) -> io::Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

fn read_i32<R: Read>(r: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_f32<R: Read>(r: &mut R) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f64<R: Read>(r: &mut R) -> io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

fn write_u8<W: Write>(w: &mut W, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

fn write_i8<W: Write>(w: &mut W, v: i8) -> io::Result<()> {
    w.write_all(&[v as u8])
}

fn write_i16<W: Write>(w: &mut W, v: i16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_i32<W: Write>(w: &mut W, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_u32<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_f32<W: Write>(w: &mut W, v: f32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_f64<W: Write>(w: &mut W, v: f64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn read_vec_f32<R: Read>(r: &mut R) -> Result<Vec<f32>> {
    let count = read_i32(r)?;
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_f32(r)?);
    }
    Ok(v)
}

fn write_vec_f32<W: Write>(w: &mut W, v: &[f32]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_f32(w, x)?;
    }
    Ok(())
}

fn read_vec_i32<R: Read>(r: &mut R) -> Result<Vec<i32>> {
    let count = read_i32(r)?;
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_i32(r)?);
    }
    Ok(v)
}

fn write_vec_i32<W: Write>(w: &mut W, v: &[i32]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_i32(w, x)?;
    }
    Ok(())
}

fn read_vec_i16<R: Read>(r: &mut R) -> Result<Vec<i16>> {
    let count = read_i32(r)?;
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_i16(r)?);
    }
    Ok(v)
}

fn write_vec_i16<W: Write>(w: &mut W, v: &[i16]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_i16(w, x)?;
    }
    Ok(())
}

trait SngRead: Sized {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self>;
}

trait SngWrite {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()>;
}

fn read_array<T: SngRead, R: Read>(r: &mut R) -> Result<Vec<T>> {
    let count = read_i32(r)?;
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(T::sng_read(r)?);
    }
    Ok(v)
}

fn write_array<T: SngWrite, W: Write>(w: &mut W, v: &[T]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for x in v {
        x.sng_write(w)?;
    }
    Ok(())
}

// --- Structs ---

#[derive(Debug, Clone, Default)]
pub struct Beat {
    pub time: f32,
    pub measure: i16,
    pub beat: i16,
    pub phrase_iteration: i32,
    pub mask: BeatMask,
}

impl Default for BeatMask {
    fn default() -> Self {
        BeatMask::empty()
    }
}

impl SngRead for Beat {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Beat {
            time: read_f32(r)?,
            measure: read_i16(r)?,
            beat: read_i16(r)?,
            phrase_iteration: read_i32(r)?,
            mask: BeatMask::from_bits_truncate(read_i32(r)?),
        })
    }
}

impl SngWrite for Beat {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        write_i16(w, self.measure)?;
        write_i16(w, self.beat)?;
        write_i32(w, self.phrase_iteration)?;
        write_i32(w, self.mask.bits())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Phrase {
    pub solo: i8,
    pub disparity: i8,
    pub ignore: i8,
    pub max_difficulty: i32,
    pub iteration_count: i32,
    pub name: [u8; 32],
}

impl Default for Phrase {
    fn default() -> Self {
        Phrase {
            solo: 0,
            disparity: 0,
            ignore: 0,
            max_difficulty: 0,
            iteration_count: 0,
            name: [0u8; 32],
        }
    }
}

impl SngRead for Phrase {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let solo = read_i8(r)?;
        let disparity = read_i8(r)?;
        let ignore = read_i8(r)?;
        let _ = read_u8(r)?; // padding
        let max_difficulty = read_i32(r)?;
        let iteration_count = read_i32(r)?;
        let mut name = [0u8; 32];
        r.read_exact(&mut name)?;
        Ok(Phrase { solo, disparity, ignore, max_difficulty, iteration_count, name })
    }
}

impl SngWrite for Phrase {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i8(w, self.solo)?;
        write_i8(w, self.disparity)?;
        write_i8(w, self.ignore)?;
        write_u8(w, 0)?; // padding
        write_i32(w, self.max_difficulty)?;
        write_i32(w, self.iteration_count)?;
        w.write_all(&self.name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhraseExtraInfo {
    pub phrase_id: i32,
    pub difficulty: i32,
    pub empty: i32,
    pub level_jump: i8,
    pub redundant: i16,
}

impl SngRead for PhraseExtraInfo {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let phrase_id = read_i32(r)?;
        let difficulty = read_i32(r)?;
        let empty = read_i32(r)?;
        let level_jump = read_i8(r)?;
        let redundant = read_i16(r)?;
        let _ = read_u8(r)?; // padding
        Ok(PhraseExtraInfo { phrase_id, difficulty, empty, level_jump, redundant })
    }
}

impl SngWrite for PhraseExtraInfo {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i32(w, self.phrase_id)?;
        write_i32(w, self.difficulty)?;
        write_i32(w, self.empty)?;
        write_i8(w, self.level_jump)?;
        write_i16(w, self.redundant)?;
        write_u8(w, 0)?; // padding
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhraseIteration {
    pub phrase_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub difficulty: [i32; 3],
}

impl SngRead for PhraseIteration {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let phrase_id = read_i32(r)?;
        let start_time = read_f32(r)?;
        let end_time = read_f32(r)?;
        let difficulty = [read_i32(r)?, read_i32(r)?, read_i32(r)?];
        Ok(PhraseIteration { phrase_id, start_time, end_time, difficulty })
    }
}

impl SngWrite for PhraseIteration {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i32(w, self.phrase_id)?;
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_i32(w, self.difficulty[0])?;
        write_i32(w, self.difficulty[1])?;
        write_i32(w, self.difficulty[2])?;
        Ok(())
    }
}

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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
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
        Ok(Section {
            name,
            number,
            start_time,
            end_time,
            start_phrase_iteration_id,
            end_phrase_iteration_id,
            string_mask,
        })
    }
}

impl SngWrite for Section {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
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

#[derive(Debug, Clone)]
pub struct Chord {
    pub mask: ChordMask,
    pub frets: [i8; 6],
    pub fingers: [i8; 6],
    pub notes: [i32; 6],
    pub name: [u8; 32],
}

impl Default for Chord {
    fn default() -> Self {
        Chord {
            mask: ChordMask::empty(),
            frets: [0i8; 6],
            fingers: [0i8; 6],
            notes: [0i32; 6],
            name: [0u8; 32],
        }
    }
}

impl Default for ChordMask {
    fn default() -> Self {
        ChordMask::empty()
    }
}

impl SngRead for Chord {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
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
        Ok(Chord { mask, frets, fingers, notes, name })
    }
}

impl SngWrite for Chord {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
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

#[derive(Debug, Clone, Default)]
pub struct BendValue {
    pub time: f32,
    pub step: f32,
    pub unused: u32,
}

impl SngRead for BendValue {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let time = read_f32(r)?;
        let step = read_f32(r)?;
        let unused = read_u32(r)?;
        Ok(BendValue { time, step, unused })
    }
}

impl SngWrite for BendValue {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        write_f32(w, self.step)?;
        write_u32(w, 0)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BendData32 {
    pub bend_values: [BendValue; 32],
    pub used_count: i32,
}

impl Default for BendData32 {
    fn default() -> Self {
        BendData32 {
            bend_values: std::array::from_fn(|_| BendValue::default()),
            used_count: 0,
        }
    }
}

impl SngRead for BendData32 {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let mut bend_values: [BendValue; 32] = std::array::from_fn(|_| BendValue::default());
        for v in bend_values.iter_mut() {
            *v = BendValue::sng_read(r)?;
        }
        let used_count = read_i32(r)?;
        Ok(BendData32 { bend_values, used_count })
    }
}

impl SngWrite for BendData32 {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        for v in &self.bend_values {
            v.sng_write(w)?;
        }
        write_i32(w, self.used_count)?;
        Ok(())
    }
}

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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let mut mask = [0u32; 6];
        for v in mask.iter_mut() {
            *v = read_u32(r)?;
        }
        let mut bend_data: [BendData32; 6] = std::array::from_fn(|_| BendData32::default());
        for v in bend_data.iter_mut() {
            *v = BendData32::sng_read(r)?;
        }
        let mut slide_to = [0i8; 6];
        for v in slide_to.iter_mut() {
            *v = read_i8(r)?;
        }
        let mut slide_unpitch_to = [0i8; 6];
        for v in slide_unpitch_to.iter_mut() {
            *v = read_i8(r)?;
        }
        let mut vibrato = [0i16; 6];
        for v in vibrato.iter_mut() {
            *v = read_i16(r)?;
        }
        Ok(ChordNotes { mask, bend_data, slide_to, slide_unpitch_to, vibrato })
    }
}

impl SngWrite for ChordNotes {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        for &v in &self.mask {
            write_u32(w, v)?;
        }
        for v in &self.bend_data {
            v.sng_write(w)?;
        }
        for &v in &self.slide_to {
            write_i8(w, v)?;
        }
        for &v in &self.slide_unpitch_to {
            write_i8(w, v)?;
        }
        for &v in &self.vibrato {
            write_i16(w, v)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Vocal {
    pub time: f32,
    pub note: i32,
    pub length: f32,
    pub lyric: [u8; 48],
}

impl Default for Vocal {
    fn default() -> Self {
        Vocal { time: 0.0, note: 0, length: 0.0, lyric: [0u8; 48] }
    }
}

impl SngRead for Vocal {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let time = read_f32(r)?;
        let note = read_i32(r)?;
        let length = read_f32(r)?;
        let mut lyric = [0u8; 48];
        r.read_exact(&mut lyric)?;
        Ok(Vocal { time, note, length, lyric })
    }
}

impl SngWrite for Vocal {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.note)?;
        write_f32(w, self.length)?;
        w.write_all(&self.lyric)?;
        Ok(())
    }
}

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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
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
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let mut font = [0u8; 128];
        r.read_exact(&mut font)?;
        let font_path_length = read_i32(r)?;
        let _ = read_i32(r)?; // unknown, written as 0
        let width = read_i32(r)?;
        let height = read_i32(r)?;
        Ok(SymbolsTexture { font, font_path_length, width, height })
    }
}

impl SngWrite for SymbolsTexture {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Rect { ymin: read_f32(r)?, xmin: read_f32(r)?, ymax: read_f32(r)?, xmax: read_f32(r)? })
    }
}

impl SngWrite for Rect {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.ymin)?;
        write_f32(w, self.xmin)?;
        write_f32(w, self.ymax)?;
        write_f32(w, self.xmax)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub symbol: [u8; 12],
    pub outer: Rect,
    pub inner: Rect,
}

impl Default for SymbolDefinition {
    fn default() -> Self {
        SymbolDefinition { symbol: [0u8; 12], outer: Rect::default(), inner: Rect::default() }
    }
}

impl SngRead for SymbolDefinition {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let mut symbol = [0u8; 12];
        r.read_exact(&mut symbol)?;
        let outer = Rect::sng_read(r)?;
        let inner = Rect::sng_read(r)?;
        Ok(SymbolDefinition { symbol, outer, inner })
    }
}

impl SngWrite for SymbolDefinition {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.symbol)?;
        self.outer.sng_write(w)?;
        self.inner.sng_write(w)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct NewLinkedDifficulty {
    pub level_break: i32,
    pub nld_phrases: Vec<i32>,
}

impl SngRead for NewLinkedDifficulty {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let level_break = read_i32(r)?;
        let nld_phrases = read_vec_i32(r)?;
        Ok(NewLinkedDifficulty { level_break, nld_phrases })
    }
}

impl SngWrite for NewLinkedDifficulty {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i32(w, self.level_break)?;
        write_vec_i32(w, &self.nld_phrases)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    pub time: f32,
    pub action_name: [u8; 256],
}

impl Default for Action {
    fn default() -> Self {
        Action { time: 0.0, action_name: [0u8; 256] }
    }
}

impl SngRead for Action {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let time = read_f32(r)?;
        let mut action_name = [0u8; 256];
        r.read_exact(&mut action_name)?;
        Ok(Action { time, action_name })
    }
}

impl SngWrite for Action {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        w.write_all(&self.action_name)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub time: f32,
    pub name: [u8; 256],
}

impl Default for Event {
    fn default() -> Self {
        Event { time: 0.0, name: [0u8; 256] }
    }
}

impl SngRead for Event {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let time = read_f32(r)?;
        let mut name = [0u8; 256];
        r.read_exact(&mut name)?;
        Ok(Event { time, name })
    }
}

impl SngWrite for Event {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        w.write_all(&self.name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tone {
    pub time: f32,
    pub tone_id: i32,
}

impl SngRead for Tone {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Tone { time: read_f32(r)?, tone_id: read_i32(r)? })
    }
}

impl SngWrite for Tone {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.tone_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DNA {
    pub time: f32,
    pub dna_id: i32,
}

impl SngRead for DNA {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        Ok(DNA { time: read_f32(r)?, dna_id: read_i32(r)? })
    }
}

impl SngWrite for DNA {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.time)?;
        write_i32(w, self.dna_id)?;
        Ok(())
    }
}

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
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
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
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
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

#[derive(Debug, Clone, Default)]
pub struct AnchorExtension {
    pub beat_time: f32,
    pub fret_id: i8,
}

impl SngRead for AnchorExtension {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let beat_time = read_f32(r)?;
        let fret_id = read_i8(r)?;
        // pad: 4 + 2 + 1 = 7 bytes
        let mut pad = [0u8; 7];
        r.read_exact(&mut pad)?;
        Ok(AnchorExtension { beat_time, fret_id })
    }
}

impl SngWrite for AnchorExtension {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f32(w, self.beat_time)?;
        write_i8(w, self.fret_id)?;
        w.write_all(&[0u8; 7])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct FingerPrint {
    pub chord_id: i32,
    pub start_time: f32,
    pub end_time: f32,
    pub first_note_time: f32,
    pub last_note_time: f32,
}

impl SngRead for FingerPrint {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
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
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i32(w, self.chord_id)?;
        write_f32(w, self.start_time)?;
        write_f32(w, self.end_time)?;
        write_f32(w, self.first_note_time)?;
        write_f32(w, self.last_note_time)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub mask: NoteMask,
    pub flags: u32,
    pub hash: u32,
    pub time: f32,
    pub string_index: i8,
    pub fret: i8,
    pub anchor_fret: i8,
    pub anchor_width: i8,
    pub chord_id: i32,
    pub chord_notes_id: i32,
    pub phrase_id: i32,
    pub phrase_iteration_id: i32,
    pub finger_print_id: [i16; 2],
    pub next_iter_note: i16,
    pub prev_iter_note: i16,
    pub parent_prev_note: i16,
    pub slide_to: i8,
    pub slide_unpitch_to: i8,
    pub left_hand: i8,
    pub tap: i8,
    pub pick_direction: i8,
    pub slap: i8,
    pub pluck: i8,
    pub vibrato: i16,
    pub sustain: f32,
    pub max_bend: f32,
    pub bend_data: Vec<BendValue>,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            mask: NoteMask::empty(),
            flags: 0,
            hash: 0,
            time: 0.0,
            string_index: 0,
            fret: 0,
            anchor_fret: 0,
            anchor_width: 0,
            chord_id: -1,
            chord_notes_id: -1,
            phrase_id: 0,
            phrase_iteration_id: 0,
            finger_print_id: [-1; 2],
            next_iter_note: -1,
            prev_iter_note: -1,
            parent_prev_note: -1,
            slide_to: -1,
            slide_unpitch_to: -1,
            left_hand: -1,
            tap: 0,
            pick_direction: 0,
            slap: -1,
            pluck: -1,
            vibrato: 0,
            sustain: 0.0,
            max_bend: 0.0,
            bend_data: vec![],
        }
    }
}

impl Default for NoteMask {
    fn default() -> Self {
        NoteMask::empty()
    }
}

impl SngRead for Note {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let mask = NoteMask::from_bits_truncate(read_u32(r)?);
        let flags = read_u32(r)?;
        let hash = read_u32(r)?;
        let time = read_f32(r)?;
        let string_index = read_i8(r)?;
        let fret = read_i8(r)?;
        let anchor_fret = read_i8(r)?;
        let anchor_width = read_i8(r)?;
        let chord_id = read_i32(r)?;
        let chord_notes_id = read_i32(r)?;
        let phrase_id = read_i32(r)?;
        let phrase_iteration_id = read_i32(r)?;
        let finger_print_id = [read_i16(r)?, read_i16(r)?];
        let next_iter_note = read_i16(r)?;
        let prev_iter_note = read_i16(r)?;
        let parent_prev_note = read_i16(r)?;
        let slide_to = read_i8(r)?;
        let slide_unpitch_to = read_i8(r)?;
        let left_hand = read_i8(r)?;
        let tap = read_i8(r)?;
        let pick_direction = read_i8(r)?;
        let slap = read_i8(r)?;
        let pluck = read_i8(r)?;
        let vibrato = read_i16(r)?;
        let sustain = read_f32(r)?;
        let max_bend = read_f32(r)?;
        let bend_data = read_array::<BendValue, _>(r)?;
        Ok(Note {
            mask,
            flags,
            hash,
            time,
            string_index,
            fret,
            anchor_fret,
            anchor_width,
            chord_id,
            chord_notes_id,
            phrase_id,
            phrase_iteration_id,
            finger_print_id,
            next_iter_note,
            prev_iter_note,
            parent_prev_note,
            slide_to,
            slide_unpitch_to,
            left_hand,
            tap,
            pick_direction,
            slap,
            pluck,
            vibrato,
            sustain,
            max_bend,
            bend_data,
        })
    }
}

impl SngWrite for Note {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u32(w, self.mask.bits())?;
        write_u32(w, self.flags)?;
        write_u32(w, self.hash)?;
        write_f32(w, self.time)?;
        write_i8(w, self.string_index)?;
        write_i8(w, self.fret)?;
        write_i8(w, self.anchor_fret)?;
        write_i8(w, self.anchor_width)?;
        write_i32(w, self.chord_id)?;
        write_i32(w, self.chord_notes_id)?;
        write_i32(w, self.phrase_id)?;
        write_i32(w, self.phrase_iteration_id)?;
        write_i16(w, self.finger_print_id[0])?;
        write_i16(w, self.finger_print_id[1])?;
        write_i16(w, self.next_iter_note)?;
        write_i16(w, self.prev_iter_note)?;
        write_i16(w, self.parent_prev_note)?;
        write_i8(w, self.slide_to)?;
        write_i8(w, self.slide_unpitch_to)?;
        write_i8(w, self.left_hand)?;
        write_i8(w, self.tap)?;
        write_i8(w, self.pick_direction)?;
        write_i8(w, self.slap)?;
        write_i8(w, self.pluck)?;
        write_i16(w, self.vibrato)?;
        write_f32(w, self.sustain)?;
        write_f32(w, self.max_bend)?;
        write_array(w, &self.bend_data)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Level {
    pub difficulty: i32,
    pub anchors: Vec<Anchor>,
    pub anchor_extensions: Vec<AnchorExtension>,
    pub hand_shapes: Vec<FingerPrint>,
    pub arpeggios: Vec<FingerPrint>,
    pub notes: Vec<Note>,
    pub average_notes_per_iteration: Vec<f32>,
    pub notes_in_phrase_iterations_excl_ignored: Vec<i32>,
    pub notes_in_phrase_iterations_all: Vec<i32>,
}

impl SngRead for Level {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let difficulty = read_i32(r)?;
        let anchors = read_array::<Anchor, _>(r)?;
        let anchor_extensions = read_array::<AnchorExtension, _>(r)?;
        let hand_shapes = read_array::<FingerPrint, _>(r)?;
        let arpeggios = read_array::<FingerPrint, _>(r)?;
        let notes = read_array::<Note, _>(r)?;
        let average_notes_per_iteration = read_vec_f32(r)?;
        let notes_in_phrase_iterations_excl_ignored = read_vec_i32(r)?;
        let notes_in_phrase_iterations_all = read_vec_i32(r)?;
        Ok(Level {
            difficulty,
            anchors,
            anchor_extensions,
            hand_shapes,
            arpeggios,
            notes,
            average_notes_per_iteration,
            notes_in_phrase_iterations_excl_ignored,
            notes_in_phrase_iterations_all,
        })
    }
}

impl SngWrite for Level {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_i32(w, self.difficulty)?;
        write_array(w, &self.anchors)?;
        write_array(w, &self.anchor_extensions)?;
        write_array(w, &self.hand_shapes)?;
        write_array(w, &self.arpeggios)?;
        write_array(w, &self.notes)?;
        write_vec_f32(w, &self.average_notes_per_iteration)?;
        write_vec_i32(w, &self.notes_in_phrase_iterations_excl_ignored)?;
        write_vec_i32(w, &self.notes_in_phrase_iterations_all)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetaData {
    pub max_score: f64,
    pub max_notes_and_chords: f64,
    pub max_notes_and_chords_real: f64,
    pub points_per_note: f64,
    pub first_beat_length: f32,
    pub start_time: f32,
    pub capo_fret_id: i8,
    pub last_conversion_date_time: [u8; 32],
    pub part: i16,
    pub song_length: f32,
    pub tuning: Vec<i16>,
    pub first_note_time: f32,
    pub max_difficulty: i32,
}

impl SngRead for MetaData {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self> {
        let max_score = read_f64(r)?;
        let max_notes_and_chords = read_f64(r)?;
        let max_notes_and_chords_real = read_f64(r)?;
        let points_per_note = read_f64(r)?;
        let first_beat_length = read_f32(r)?;
        let start_time = read_f32(r)?;
        let capo_fret_id = read_i8(r)?;
        let mut last_conversion_date_time = [0u8; 32];
        r.read_exact(&mut last_conversion_date_time)?;
        let part = read_i16(r)?;
        let song_length = read_f32(r)?;
        let tuning = read_vec_i16(r)?;
        let first_note_time = read_f32(r)?;
        let _ = read_f32(r)?; // second copy
        let max_difficulty = read_i32(r)?;
        Ok(MetaData {
            max_score,
            max_notes_and_chords,
            max_notes_and_chords_real,
            points_per_note,
            first_beat_length,
            start_time,
            capo_fret_id,
            last_conversion_date_time,
            part,
            song_length,
            tuning,
            first_note_time,
            max_difficulty,
        })
    }
}

impl SngWrite for MetaData {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_f64(w, self.max_score)?;
        write_f64(w, self.max_notes_and_chords)?;
        write_f64(w, self.max_notes_and_chords_real)?;
        write_f64(w, self.points_per_note)?;
        write_f32(w, self.first_beat_length)?;
        write_f32(w, self.start_time)?;
        write_i8(w, self.capo_fret_id)?;
        w.write_all(&self.last_conversion_date_time)?;
        write_i16(w, self.part)?;
        write_f32(w, self.song_length)?;
        write_vec_i16(w, &self.tuning)?;
        write_f32(w, self.first_note_time)?;
        write_f32(w, self.first_note_time)?; // written twice
        write_i32(w, self.max_difficulty)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sng {
    pub beats: Vec<Beat>,
    pub phrases: Vec<Phrase>,
    pub chords: Vec<Chord>,
    pub chord_notes: Vec<ChordNotes>,
    pub vocals: Vec<Vocal>,
    pub symbols_headers: Vec<SymbolsHeader>,
    pub symbols_textures: Vec<SymbolsTexture>,
    pub symbol_definitions: Vec<SymbolDefinition>,
    pub phrase_iterations: Vec<PhraseIteration>,
    pub phrase_extra_info: Vec<PhraseExtraInfo>,
    pub new_linked_difficulties: Vec<NewLinkedDifficulty>,
    pub actions: Vec<Action>,
    pub events: Vec<Event>,
    pub tones: Vec<Tone>,
    pub dnas: Vec<DNA>,
    pub sections: Vec<Section>,
    pub levels: Vec<Level>,
    pub metadata: MetaData,
}

impl Sng {
    pub fn read(data: &[u8]) -> Result<Self> {
        let mut r = std::io::Cursor::new(data);
        let beats = read_array::<Beat, _>(&mut r)?;
        let phrases = read_array::<Phrase, _>(&mut r)?;
        let chords = read_array::<Chord, _>(&mut r)?;
        let chord_notes = read_array::<ChordNotes, _>(&mut r)?;
        let vocals = read_array::<Vocal, _>(&mut r)?;
        let (symbols_headers, symbols_textures, symbol_definitions) = if !vocals.is_empty() {
            (
                read_array::<SymbolsHeader, _>(&mut r)?,
                read_array::<SymbolsTexture, _>(&mut r)?,
                read_array::<SymbolDefinition, _>(&mut r)?,
            )
        } else {
            (vec![], vec![], vec![])
        };
        let phrase_iterations = read_array::<PhraseIteration, _>(&mut r)?;
        let phrase_extra_info = read_array::<PhraseExtraInfo, _>(&mut r)?;
        let new_linked_difficulties = read_array::<NewLinkedDifficulty, _>(&mut r)?;
        let actions = read_array::<Action, _>(&mut r)?;
        let events = read_array::<Event, _>(&mut r)?;
        let tones = read_array::<Tone, _>(&mut r)?;
        let dnas = read_array::<DNA, _>(&mut r)?;
        let sections = read_array::<Section, _>(&mut r)?;
        let levels = read_array::<Level, _>(&mut r)?;
        let metadata = MetaData::sng_read(&mut r)?;
        Ok(Sng {
            beats,
            phrases,
            chords,
            chord_notes,
            vocals,
            symbols_headers,
            symbols_textures,
            symbol_definitions,
            phrase_iterations,
            phrase_extra_info,
            new_linked_difficulties,
            actions,
            events,
            tones,
            dnas,
            sections,
            levels,
            metadata,
        })
    }

    pub fn write(&self) -> Result<Vec<u8>> {
        let mut w = Vec::new();
        write_array(&mut w, &self.beats)?;
        write_array(&mut w, &self.phrases)?;
        write_array(&mut w, &self.chords)?;
        write_array(&mut w, &self.chord_notes)?;
        write_array(&mut w, &self.vocals)?;
        if !self.vocals.is_empty() {
            write_array(&mut w, &self.symbols_headers)?;
            write_array(&mut w, &self.symbols_textures)?;
            write_array(&mut w, &self.symbol_definitions)?;
        }
        write_array(&mut w, &self.phrase_iterations)?;
        write_array(&mut w, &self.phrase_extra_info)?;
        write_array(&mut w, &self.new_linked_difficulties)?;
        write_array(&mut w, &self.actions)?;
        write_array(&mut w, &self.events)?;
        write_array(&mut w, &self.tones)?;
        write_array(&mut w, &self.dnas)?;
        write_array(&mut w, &self.sections)?;
        write_array(&mut w, &self.levels)?;
        self.metadata.sng_write(&mut w)?;
        Ok(w)
    }

    pub fn from_encrypted(data: &[u8], platform: Platform) -> Result<Self> {
        let decrypted = decrypt_sng(data, platform)?;
        Sng::read(&decrypted)
    }

    pub fn to_encrypted(&self, platform: Platform) -> Result<Vec<u8>> {
        let raw = self.write()?;
        encrypt_sng(&raw, platform)
    }
}

pub fn decrypt_sng(input: &[u8], platform: Platform) -> Result<Vec<u8>> {
    if input.len() < 24 || &input[0..4] != &[0x4A, 0, 0, 0] {
        return Err(Error::InvalidHeader);
    }
    let iv = &input[8..24];
    let key = match platform {
        Platform::Pc => &PC_KEY,
        Platform::Mac => &MAC_KEY,
    };
    let mut payload = input[24..].to_vec();
    let mut cipher = AesCtr::new_from_slices(key, iv).map_err(|_| Error::Crypto)?;
    cipher.apply_keystream(&mut payload);

    if payload.len() < 4 {
        return Err(Error::InvalidHeader);
    }
    let compressed = &payload[4..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

pub fn encrypt_sng(data: &[u8], platform: Platform) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;

    let mut payload = Vec::with_capacity(4 + compressed.len());
    payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
    payload.extend_from_slice(&compressed);

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    let key = match platform {
        Platform::Pc => &PC_KEY,
        Platform::Mac => &MAC_KEY,
    };
    let mut cipher = AesCtr::new_from_slices(key, &iv).map_err(|_| Error::Crypto)?;
    cipher.apply_keystream(&mut payload);

    let mut out = Vec::new();
    out.extend_from_slice(&[0x4A, 0, 0, 0]);
    out.extend_from_slice(&[3, 0, 0, 0]);
    out.extend_from_slice(&iv);
    out.extend_from_slice(&payload);
    out.extend_from_slice(&[0u8; 56]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bend_value_roundtrip() {
        let bv = BendValue { time: 1.5, step: 0.5, unused: 0 };
        let mut buf = Vec::new();
        bv.sng_write(&mut buf).unwrap();
        assert_eq!(buf.len(), 12);
        let bv2 = BendValue::sng_read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(bv.time, bv2.time);
        assert_eq!(bv.step, bv2.step);
    }

    #[test]
    fn test_beat_roundtrip() {
        let beat = Beat {
            time: 1.0,
            measure: 1,
            beat: 0,
            phrase_iteration: 0,
            mask: BeatMask::FIRST_BEAT_OF_MEASURE,
        };
        let mut buf = Vec::new();
        beat.sng_write(&mut buf).unwrap();
        let beat2 = Beat::sng_read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(beat.time, beat2.time);
        assert_eq!(beat.mask, beat2.mask);
    }

    #[test]
    fn test_note_roundtrip() {
        let note = Note {
            mask: NoteMask::SINGLE,
            flags: 0,
            hash: 0,
            time: 2.5,
            string_index: 2,
            fret: 5,
            anchor_fret: 5,
            anchor_width: 4,
            chord_id: -1,
            chord_notes_id: -1,
            phrase_id: 0,
            phrase_iteration_id: 0,
            finger_print_id: [-1, -1],
            next_iter_note: -1,
            prev_iter_note: -1,
            parent_prev_note: -1,
            slide_to: -1,
            slide_unpitch_to: -1,
            left_hand: -1,
            tap: 0,
            pick_direction: 0,
            slap: -1,
            pluck: -1,
            vibrato: 0,
            sustain: 0.0,
            max_bend: 0.0,
            bend_data: vec![],
        };
        let mut buf = Vec::new();
        note.sng_write(&mut buf).unwrap();
        let note2 = Note::sng_read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(note.mask, note2.mask);
        assert_eq!(note.fret, note2.fret);
        assert!(note2.bend_data.is_empty());
    }

    #[test]
    fn test_level_roundtrip() {
        let level = Level {
            difficulty: 0,
            anchors: vec![Anchor {
                start_time: 0.0,
                end_time: 10.0,
                first_note_time: 0.5,
                last_note_time: 9.5,
                fret_id: 1,
                width: 4,
                phrase_iteration_id: 0,
            }],
            anchor_extensions: vec![],
            hand_shapes: vec![],
            arpeggios: vec![],
            notes: vec![],
            average_notes_per_iteration: vec![1.0],
            notes_in_phrase_iterations_excl_ignored: vec![1],
            notes_in_phrase_iterations_all: vec![1],
        };
        let mut buf = Vec::new();
        level.sng_write(&mut buf).unwrap();
        let level2 = Level::sng_read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(level.difficulty, level2.difficulty);
        assert_eq!(level.anchors.len(), level2.anchors.len());
        assert_eq!(level.anchors[0].fret_id, level2.anchors[0].fret_id);
    }

    #[test]
    fn test_empty_sng_roundtrip() {
        let sng = Sng::default();
        let bytes = sng.write().unwrap();
        let sng2 = Sng::read(&bytes).unwrap();
        assert_eq!(sng2.beats.len(), 0);
        assert_eq!(sng2.levels.len(), 0);
    }

    #[test]
    fn test_encrypted_roundtrip() {
        let sng = Sng::default();
        let enc = sng.to_encrypted(Platform::Pc).unwrap();
        let sng2 = Sng::from_encrypted(&enc, Platform::Pc).unwrap();
        assert_eq!(sng2.beats.len(), 0);
        assert_eq!(sng2.metadata.max_score, 0.0);
    }
}
