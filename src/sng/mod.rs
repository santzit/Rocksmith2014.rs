pub mod types;
pub use types::Sng;

use std::{
    io::{self, Cursor, Read},
    path::Path,
};

use aes::Aes256;
use ctr::{Ctr128BE, cipher::{KeyIvInit, StreamCipher}};
use flate2::read::ZlibDecoder;

use crate::error::{Error, Result};
use types::*;

// ---------------------------------------------------------------------------
// Encryption keys
// ---------------------------------------------------------------------------

/// AES-256 key for PC SNG files.
const SNG_KEY_PC: [u8; 32] = [
    0xCB, 0x64, 0x8D, 0xF3, 0xD1, 0x2A, 0x16, 0xBF,
    0x71, 0x70, 0x14, 0x14, 0xE6, 0x96, 0x19, 0xEC,
    0x17, 0x1C, 0xCA, 0x5D, 0x2A, 0x14, 0x2E, 0x3E,
    0x59, 0xDE, 0x7A, 0xDD, 0xA1, 0x8A, 0x3A, 0x30,
];

/// AES-256 key for Mac SNG files.
const SNG_KEY_MAC: [u8; 32] = [
    0x98, 0x21, 0x33, 0x0E, 0x34, 0xB9, 0x1F, 0x70,
    0xD0, 0xA4, 0x8C, 0xBD, 0x62, 0x59, 0x93, 0x12,
    0x69, 0x70, 0xCE, 0xA0, 0x91, 0x92, 0xC0, 0xE6,
    0xCD, 0xA6, 0x76, 0xCC, 0x98, 0x38, 0x28, 0x9D,
];

/// Platform selector for SNG decryption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Platform {
    Pc = 0,
    Mac = 1,
}

impl Platform {
    pub fn from_i32(v: i32) -> Self {
        if v == 1 { Platform::Mac } else { Platform::Pc }
    }

    fn key(self) -> &'static [u8; 32] {
        match self {
            Platform::Pc => &SNG_KEY_PC,
            Platform::Mac => &SNG_KEY_MAC,
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

impl Sng {
    /// Read an encrypted (packed) SNG from `path`.
    pub fn read_packed_file(path: impl AsRef<Path>, platform: Platform) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_packed_bytes(&data, platform)
    }

    /// Read an unencrypted (plain) SNG from `path`.
    pub fn read_unpacked_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_unpacked_bytes(&data)
    }

    /// Decrypt and decompress packed SNG bytes, then parse.
    pub fn from_packed_bytes(data: &[u8], platform: Platform) -> Result<Self> {
        let mut cur = Cursor::new(data);
        let decrypted = decrypt_sng(&mut cur, platform)?;
        Self::from_unpacked_bytes(&decrypted)
    }

    /// Parse raw (plaintext) SNG bytes.
    pub fn from_unpacked_bytes(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        parse_sng(&mut r)
    }
}

// ---------------------------------------------------------------------------
// Decryption
// ---------------------------------------------------------------------------

/// Decrypt an encrypted SNG stream into raw (uncompressed) bytes.
fn decrypt_sng(cur: &mut Cursor<&[u8]>, platform: Platform) -> Result<Vec<u8>> {
    // magic (u32), header (u32)
    let magic = read_u32_le(cur)?;
    if magic != 0x4A {
        return Err(Error::InvalidSng(format!("bad magic: 0x{magic:08X}")));
    }
    let _header = read_u32_le(cur)?;

    // IV
    let mut iv = [0u8; 16];
    cur.read_exact(&mut iv)?;

    // Encrypted payload (everything after magic + header + IV)
    let pos = cur.position() as usize;
    let payload = &cur.get_ref()[pos..];

    // AES-256-CTR decrypt
    let key = platform.key();
    let mut cipher = Ctr128BE::<Aes256>::new(key.into(), &iv.into());
    let mut decrypted = payload.to_vec();
    cipher.apply_keystream(&mut decrypted);

    // First 4 bytes = uncompressed length; remainder = zlib data
    if decrypted.len() < 4 {
        return Err(Error::InvalidSng("too short after decryption".into()));
    }
    let plain_len =
        u32::from_le_bytes(decrypted[..4].try_into().unwrap()) as usize;
    let compressed = &decrypted[4..];

    let mut out = Vec::with_capacity(plain_len);
    let mut dec = ZlibDecoder::new(compressed);
    dec.read_to_end(&mut out)
        .map_err(|e| Error::InvalidSng(format!("zlib: {e}")))?;

    Ok(out)
}

// ---------------------------------------------------------------------------
// Binary parsing
// ---------------------------------------------------------------------------

/// Thin wrapper around a byte slice providing little-endian reads.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let end = self.pos + buf.len();
        if end > self.data.len() {
            return Err(Error::InvalidSng("unexpected end of data".into()));
        }
        buf.copy_from_slice(&self.data[self.pos..end]);
        self.pos = end;
        Ok(())
    }

    fn read_i8(&mut self) -> Result<i8> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0] as i8)
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut b = [0u8; 2];
        self.read_exact(&mut b)?;
        Ok(i16::from_le_bytes(b))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(i32::from_le_bytes(b))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(f32::from_le_bytes(b))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let mut b = [0u8; 8];
        self.read_exact(&mut b)?;
        Ok(f64::from_le_bytes(b))
    }

    /// Read a zero-terminated UTF-8 string padded to `len` bytes.
    fn read_string(&mut self, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
        Ok(String::from_utf8_lossy(&buf[..end]).into_owned())
    }

    /// Read an array prefixed by an i32 count.
    fn read_array<T, F>(&mut self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&mut Self) -> Result<T>,
    {
        let count = self.read_i32()? as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(f(self)?);
        }
        Ok(v)
    }
}

// ---------------------------------------------------------------------------
// SNG parser
// ---------------------------------------------------------------------------

fn parse_sng(r: &mut Reader<'_>) -> Result<Sng> {
    let beats = r.read_array(read_beat)?;
    let phrases = r.read_array(read_phrase)?;
    let chords = r.read_array(read_chord)?;
    let chord_notes = r.read_array(read_chord_notes)?;
    let vocals = r.read_array(read_vocal)?;

    let (symbols_headers, symbols_textures, symbol_definitions) =
        if !vocals.is_empty() {
            (
                r.read_array(read_symbols_header)?,
                r.read_array(read_symbols_texture)?,
                r.read_array(read_symbol_definition)?,
            )
        } else {
            (vec![], vec![], vec![])
        };

    let phrase_iterations = r.read_array(read_phrase_iteration)?;
    let phrase_extra_info = r.read_array(read_phrase_extra_info)?;
    let new_linked_difficulties = r.read_array(read_new_linked_difficulty)?;
    let actions = r.read_array(read_action)?;
    let events = r.read_array(read_event)?;
    let tones = r.read_array(read_tone)?;
    let dnas = r.read_array(read_dna)?;
    let sections = r.read_array(read_section)?;
    let levels = r.read_array(read_level)?;
    let metadata = read_metadata(r)?;

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

fn read_beat(r: &mut Reader<'_>) -> Result<Beat> {
    Ok(Beat {
        time: r.read_f32()?,
        measure: r.read_i16()?,
        beat: r.read_i16()?,
        phrase_iteration: r.read_i32()?,
        mask: BeatMask::from_bits_truncate(r.read_u32()?),
    })
}

fn read_phrase(r: &mut Reader<'_>) -> Result<Phrase> {
    let solo = r.read_i8()?;
    let disparity = r.read_i8()?;
    let ignore = r.read_i8()?;
    let _pad = r.read_i8()?; // 1 byte padding
    Ok(Phrase {
        solo,
        disparity,
        ignore,
        max_difficulty: r.read_i32()?,
        iteration_count: r.read_i32()?,
        name: r.read_string(32)?,
    })
}

fn read_chord(r: &mut Reader<'_>) -> Result<Chord> {
    let mask = ChordMask::from_bits_truncate(r.read_u32()?);
    let mut frets = [0i8; 6];
    for f in &mut frets {
        *f = r.read_i8()?;
    }
    let mut fingers = [0i8; 6];
    for f in &mut fingers {
        *f = r.read_i8()?;
    }
    let mut notes = [0i32; 6];
    for n in &mut notes {
        *n = r.read_i32()?;
    }
    Ok(Chord { mask, frets, fingers, notes, name: r.read_string(32)? })
}

fn read_bend_value(r: &mut Reader<'_>) -> Result<BendValue> {
    let time = r.read_f32()?;
    let step = r.read_f32()?;
    let _unk = r.read_i32()?; // 4 unknown bytes, always near-zero
    Ok(BendValue { time, step })
}

fn read_bend_data32(r: &mut Reader<'_>) -> Result<BendData32> {
    let mut bend_values: [BendValue; 32] =
        std::array::from_fn(|_| BendValue::default());
    for v in &mut bend_values {
        *v = read_bend_value(r)?;
    }
    let used_count = r.read_i32()?;
    Ok(BendData32 { bend_values, used_count })
}

fn read_chord_notes(r: &mut Reader<'_>) -> Result<ChordNotes> {
    let mut mask = [NoteMask::empty(); 6];
    for m in &mut mask {
        *m = NoteMask::from_bits_truncate(r.read_u32()?);
    }
    // BendData32 is not array-prefixed here; there are always 6 of them
    let mut bend_data: [BendData32; 6] = std::array::from_fn(|_| BendData32 {
        bend_values: std::array::from_fn(|_| BendValue::default()),
        used_count: 0,
    });
    for b in &mut bend_data {
        *b = read_bend_data32(r)?;
    }
    let mut slide_to = [0i8; 6];
    for s in &mut slide_to {
        *s = r.read_i8()?;
    }
    let mut slide_unpitch_to = [0i8; 6];
    for s in &mut slide_unpitch_to {
        *s = r.read_i8()?;
    }
    let mut vibrato = [0i16; 6];
    for v in &mut vibrato {
        *v = r.read_i16()?;
    }
    Ok(ChordNotes { mask, bend_data, slide_to, slide_unpitch_to, vibrato })
}

fn read_vocal(r: &mut Reader<'_>) -> Result<Vocal> {
    Ok(Vocal {
        time: r.read_f32()?,
        note: r.read_i32()?,
        length: r.read_f32()?,
        lyric: r.read_string(48)?,
    })
}

fn read_symbols_header(r: &mut Reader<'_>) -> Result<SymbolsHeader> {
    let id = r.read_i32()?;
    let mut unk = [0i32; 7];
    for u in &mut unk {
        *u = r.read_i32()?;
    }
    Ok(SymbolsHeader { id, unk })
}

fn read_symbols_texture(r: &mut Reader<'_>) -> Result<SymbolsTexture> {
    let font = r.read_string(128)?;
    let font_path_length = r.read_i32()?;
    let _unk = r.read_i32()?; // always zero
    let width = r.read_i32()?;
    let height = r.read_i32()?;
    Ok(SymbolsTexture { font, font_path_length, width, height })
}

fn read_rect(r: &mut Reader<'_>) -> Result<Rect> {
    Ok(Rect {
        y_min: r.read_f32()?,
        x_min: r.read_f32()?,
        y_max: r.read_f32()?,
        x_max: r.read_f32()?,
    })
}

fn read_symbol_definition(r: &mut Reader<'_>) -> Result<SymbolDefinition> {
    Ok(SymbolDefinition {
        symbol: r.read_string(12)?,
        outer: read_rect(r)?,
        inner: read_rect(r)?,
    })
}

fn read_phrase_iteration(r: &mut Reader<'_>) -> Result<PhraseIteration> {
    let phrase_id = r.read_i32()?;
    let start_time = r.read_f32()?;
    let end_time = r.read_f32()?;
    let mut difficulty = [0i32; 3];
    for d in &mut difficulty {
        *d = r.read_i32()?;
    }
    Ok(PhraseIteration { phrase_id, start_time, end_time, difficulty })
}

fn read_phrase_extra_info(r: &mut Reader<'_>) -> Result<PhraseExtraInfo> {
    let info = PhraseExtraInfo {
        phrase_id: r.read_i32()?,
        difficulty: r.read_i32()?,
        empty: r.read_i32()?,
        level_jump: r.read_i8()?,
        redundant: r.read_i16()?,
    };
    let _pad = r.read_i8()?; // 1 byte padding
    Ok(info)
}

fn read_new_linked_difficulty(r: &mut Reader<'_>) -> Result<NewLinkedDifficulty> {
    let level_break = r.read_i32()?;
    let nld_phrases = r.read_array(|r| r.read_i32())?;
    Ok(NewLinkedDifficulty { level_break, nld_phrases })
}

fn read_action(r: &mut Reader<'_>) -> Result<Action> {
    Ok(Action {
        time: r.read_f32()?,
        action_name: r.read_string(256)?,
    })
}

fn read_event(r: &mut Reader<'_>) -> Result<Event> {
    Ok(Event {
        time: r.read_f32()?,
        name: r.read_string(256)?,
    })
}

fn read_tone(r: &mut Reader<'_>) -> Result<Tone> {
    Ok(Tone { time: r.read_f32()?, tone_id: r.read_i32()? })
}

fn read_dna(r: &mut Reader<'_>) -> Result<Dna> {
    Ok(Dna { time: r.read_f32()?, dna_id: r.read_i32()? })
}

fn read_section(r: &mut Reader<'_>) -> Result<Section> {
    let name = r.read_string(32)?;
    let number = r.read_i32()?;
    let start_time = r.read_f32()?;
    let end_time = r.read_f32()?;
    let start_phrase_iteration_id = r.read_i32()?;
    let end_phrase_iteration_id = r.read_i32()?;
    let mut string_mask = [0i8; 36];
    for s in &mut string_mask {
        *s = r.read_i8()?;
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

fn read_anchor(r: &mut Reader<'_>) -> Result<Anchor> {
    let start_time = r.read_f32()?;
    let end_time = r.read_f32()?;
    let first_note_time = r.read_f32()?;
    let last_note_time = r.read_f32()?;
    let fret_id = r.read_i8()?;
    // 3 bytes padding
    r.read_u8()?;
    r.read_u8()?;
    r.read_u8()?;
    let width = r.read_i32()?;
    let phrase_iteration_id = r.read_i32()?;
    Ok(Anchor {
        start_time,
        end_time,
        first_note_time,
        last_note_time,
        fret_id,
        width,
        phrase_iteration_id,
    })
}

fn read_anchor_extension(r: &mut Reader<'_>) -> Result<AnchorExtension> {
    let beat_time = r.read_f32()?;
    let fret_id = r.read_i8()?;
    let _unk1 = r.read_i32()?; // always 0
    let _unk2 = r.read_i16()?; // always 0
    let _unk3 = r.read_i8()?;  // always 0
    Ok(AnchorExtension { beat_time, fret_id })
}

fn read_finger_print(r: &mut Reader<'_>) -> Result<FingerPrint> {
    Ok(FingerPrint {
        chord_id: r.read_i32()?,
        start_time: r.read_f32()?,
        end_time: r.read_f32()?,
        first_note_time: r.read_f32()?,
        last_note_time: r.read_f32()?,
    })
}

fn read_note(r: &mut Reader<'_>) -> Result<Note> {
    let mask = NoteMask::from_bits_truncate(r.read_u32()?);
    let flags = r.read_u32()?;
    let hash = r.read_u32()?;
    let time = r.read_f32()?;
    let string_index = r.read_i8()?;
    let fret = r.read_i8()?;
    let anchor_fret = r.read_i8()?;
    let anchor_width = r.read_i8()?;
    let chord_id = r.read_i32()?;
    let chord_notes_id = r.read_i32()?;
    let phrase_id = r.read_i32()?;
    let phrase_iteration_id = r.read_i32()?;
    let finger_print_id = [r.read_i16()?, r.read_i16()?];
    let next_iter_note = r.read_i16()?;
    let prev_iter_note = r.read_i16()?;
    let parent_prev_note = r.read_i16()?;
    let slide_to = r.read_i8()?;
    let slide_unpitch_to = r.read_i8()?;
    let left_hand = r.read_i8()?;
    let tap = r.read_i8()?;
    let pick_direction = r.read_i8()?;
    let slap = r.read_i8()?;
    let pluck = r.read_i8()?;
    let vibrato = r.read_i16()?;
    let sustain = r.read_f32()?;
    let max_bend = r.read_f32()?;
    let bend_data = r.read_array(read_bend_value)?;
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

fn read_level(r: &mut Reader<'_>) -> Result<Level> {
    let difficulty = r.read_i32()?;
    let anchors = r.read_array(read_anchor)?;
    let anchor_extensions = r.read_array(read_anchor_extension)?;
    let hand_shapes = r.read_array(read_finger_print)?;
    let arpeggios = r.read_array(read_finger_print)?;
    let notes = r.read_array(read_note)?;
    let average_notes_per_iteration = r.read_array(|r| r.read_f32())?;
    let notes_in_phrase_iterations_excl_ignored = r.read_array(|r| r.read_i32())?;
    let notes_in_phrase_iterations_all = r.read_array(|r| r.read_i32())?;
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

fn read_metadata(r: &mut Reader<'_>) -> Result<MetaData> {
    let max_score = r.read_f64()?;
    let max_notes_and_chords = r.read_f64()?;
    let max_notes_and_chords_real = r.read_f64()?;
    let points_per_note = r.read_f64()?;
    let first_beat_length = r.read_f32()?;
    let start_time = r.read_f32()?;
    let capo_fret_id = r.read_i8()?;
    let last_conversion_date_time = r.read_string(32)?;
    let part = r.read_i16()?;
    let song_length = r.read_f32()?;
    let tuning = r.read_array(|r| r.read_i16())?;
    let _first_note_time_dup = r.read_f32()?; // written twice in the format
    let first_note_time = r.read_f32()?;
    let max_difficulty = r.read_i32()?;
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

fn read_u32_le(cur: &mut Cursor<&[u8]>) -> io::Result<u32> {
    let mut b = [0u8; 4];
    cur.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

// ---------------------------------------------------------------------------
// SNG serialiser (used only by test-fixture generator)
// ---------------------------------------------------------------------------

/// Write `count` zero-bytes to ensure padding / fixed-length fields.
fn pad(out: &mut Vec<u8>, n: usize) {
    out.extend(std::iter::repeat(0u8).take(n));
}

fn write_i8(out: &mut Vec<u8>, v: i8) {
    out.push(v as u8);
}
fn write_i16_le(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn write_i32_le(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn write_f32_le(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn write_f64_le(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn write_fixed_str(out: &mut Vec<u8>, s: &str, len: usize) {
    let bytes = s.as_bytes();
    let copy = bytes.len().min(len.saturating_sub(1));
    out.extend_from_slice(&bytes[..copy]);
    pad(out, len - copy);
}
fn write_count(out: &mut Vec<u8>, n: usize) {
    write_i32_le(out, n as i32);
}

/// Serialise an [`Sng`] to unpacked (plain) bytes.
pub fn sng_to_bytes(sng: &Sng) -> Vec<u8> {
    let mut out = Vec::new();

    // beats
    write_count(&mut out, sng.beats.len());
    for b in &sng.beats {
        write_f32_le(&mut out, b.time);
        write_i16_le(&mut out, b.measure);
        write_i16_le(&mut out, b.beat);
        write_i32_le(&mut out, b.phrase_iteration);
        write_u32_le(&mut out, b.mask.bits());
    }

    // phrases
    write_count(&mut out, sng.phrases.len());
    for p in &sng.phrases {
        write_i8(&mut out, p.solo);
        write_i8(&mut out, p.disparity);
        write_i8(&mut out, p.ignore);
        pad(&mut out, 1); // padding
        write_i32_le(&mut out, p.max_difficulty);
        write_i32_le(&mut out, p.iteration_count);
        write_fixed_str(&mut out, &p.name, 32);
    }

    // chords
    write_count(&mut out, sng.chords.len());
    for c in &sng.chords {
        write_u32_le(&mut out, c.mask.bits());
        for f in &c.frets { write_i8(&mut out, *f); }
        for f in &c.fingers { write_i8(&mut out, *f); }
        for n in &c.notes { write_i32_le(&mut out, *n); }
        write_fixed_str(&mut out, &c.name, 32);
    }

    // chord notes
    write_count(&mut out, sng.chord_notes.len());
    for cn in &sng.chord_notes {
        for m in &cn.mask { write_u32_le(&mut out, m.bits()); }
        for bd in &cn.bend_data { write_bend_data32(&mut out, bd); }
        for s in &cn.slide_to { write_i8(&mut out, *s); }
        for s in &cn.slide_unpitch_to { write_i8(&mut out, *s); }
        for v in &cn.vibrato { write_i16_le(&mut out, *v); }
    }

    // vocals
    write_count(&mut out, sng.vocals.len());
    for v in &sng.vocals {
        write_f32_le(&mut out, v.time);
        write_i32_le(&mut out, v.note);
        write_f32_le(&mut out, v.length);
        write_fixed_str(&mut out, &v.lyric, 48);
    }

    // symbol data (only if vocals non-empty)
    if !sng.vocals.is_empty() {
        write_count(&mut out, sng.symbols_headers.len());
        for sh in &sng.symbols_headers {
            write_i32_le(&mut out, sh.id);
            for u in &sh.unk { write_i32_le(&mut out, *u); }
        }
        write_count(&mut out, sng.symbols_textures.len());
        for st in &sng.symbols_textures {
            write_fixed_str(&mut out, &st.font, 128);
            write_i32_le(&mut out, st.font_path_length);
            write_i32_le(&mut out, 0); // unknown
            write_i32_le(&mut out, st.width);
            write_i32_le(&mut out, st.height);
        }
        write_count(&mut out, sng.symbol_definitions.len());
        for sd in &sng.symbol_definitions {
            write_fixed_str(&mut out, &sd.symbol, 12);
            write_rect(&mut out, &sd.outer);
            write_rect(&mut out, &sd.inner);
        }
    }

    // phrase iterations
    write_count(&mut out, sng.phrase_iterations.len());
    for pi in &sng.phrase_iterations {
        write_i32_le(&mut out, pi.phrase_id);
        write_f32_le(&mut out, pi.start_time);
        write_f32_le(&mut out, pi.end_time);
        for d in &pi.difficulty { write_i32_le(&mut out, *d); }
    }

    // phrase extra info
    write_count(&mut out, sng.phrase_extra_info.len());
    for pe in &sng.phrase_extra_info {
        write_i32_le(&mut out, pe.phrase_id);
        write_i32_le(&mut out, pe.difficulty);
        write_i32_le(&mut out, pe.empty);
        write_i8(&mut out, pe.level_jump);
        write_i16_le(&mut out, pe.redundant);
        pad(&mut out, 1); // padding
    }

    // new linked difficulties
    write_count(&mut out, sng.new_linked_difficulties.len());
    for nld in &sng.new_linked_difficulties {
        write_i32_le(&mut out, nld.level_break);
        write_count(&mut out, nld.nld_phrases.len());
        for p in &nld.nld_phrases { write_i32_le(&mut out, *p); }
    }

    // actions
    write_count(&mut out, sng.actions.len());
    for a in &sng.actions {
        write_f32_le(&mut out, a.time);
        write_fixed_str(&mut out, &a.action_name, 256);
    }

    // events
    write_count(&mut out, sng.events.len());
    for e in &sng.events {
        write_f32_le(&mut out, e.time);
        write_fixed_str(&mut out, &e.name, 256);
    }

    // tones
    write_count(&mut out, sng.tones.len());
    for t in &sng.tones {
        write_f32_le(&mut out, t.time);
        write_i32_le(&mut out, t.tone_id);
    }

    // dnas
    write_count(&mut out, sng.dnas.len());
    for d in &sng.dnas {
        write_f32_le(&mut out, d.time);
        write_i32_le(&mut out, d.dna_id);
    }

    // sections
    write_count(&mut out, sng.sections.len());
    for s in &sng.sections {
        write_fixed_str(&mut out, &s.name, 32);
        write_i32_le(&mut out, s.number);
        write_f32_le(&mut out, s.start_time);
        write_f32_le(&mut out, s.end_time);
        write_i32_le(&mut out, s.start_phrase_iteration_id);
        write_i32_le(&mut out, s.end_phrase_iteration_id);
        for sm in &s.string_mask { write_i8(&mut out, *sm); }
    }

    // levels
    write_count(&mut out, sng.levels.len());
    for lv in &sng.levels {
        write_i32_le(&mut out, lv.difficulty);
        write_count(&mut out, lv.anchors.len());
        for a in &lv.anchors { write_anchor(&mut out, a); }
        write_count(&mut out, lv.anchor_extensions.len());
        for ae in &lv.anchor_extensions { write_anchor_extension(&mut out, ae); }
        write_count(&mut out, lv.hand_shapes.len());
        for fp in &lv.hand_shapes { write_finger_print(&mut out, fp); }
        write_count(&mut out, lv.arpeggios.len());
        for fp in &lv.arpeggios { write_finger_print(&mut out, fp); }
        write_count(&mut out, lv.notes.len());
        for n in &lv.notes { write_note(&mut out, n); }
        write_count(&mut out, lv.average_notes_per_iteration.len());
        for v in &lv.average_notes_per_iteration { write_f32_le(&mut out, *v); }
        write_count(&mut out, lv.notes_in_phrase_iterations_excl_ignored.len());
        for v in &lv.notes_in_phrase_iterations_excl_ignored {
            write_i32_le(&mut out, *v);
        }
        write_count(&mut out, lv.notes_in_phrase_iterations_all.len());
        for v in &lv.notes_in_phrase_iterations_all { write_i32_le(&mut out, *v); }
    }

    // metadata
    let md = &sng.metadata;
    write_f64_le(&mut out, md.max_score);
    write_f64_le(&mut out, md.max_notes_and_chords);
    write_f64_le(&mut out, md.max_notes_and_chords_real);
    write_f64_le(&mut out, md.points_per_note);
    write_f32_le(&mut out, md.first_beat_length);
    write_f32_le(&mut out, md.start_time);
    write_i8(&mut out, md.capo_fret_id);
    write_fixed_str(&mut out, &md.last_conversion_date_time, 32);
    write_i16_le(&mut out, md.part);
    write_f32_le(&mut out, md.song_length);
    write_count(&mut out, md.tuning.len());
    for t in &md.tuning { write_i16_le(&mut out, *t); }
    write_f32_le(&mut out, md.first_note_time); // written twice
    write_f32_le(&mut out, md.first_note_time);
    write_i32_le(&mut out, md.max_difficulty);

    out
}

fn write_bend_data32(out: &mut Vec<u8>, bd: &BendData32) {
    for bv in &bd.bend_values {
        write_f32_le(out, bv.time);
        write_f32_le(out, bv.step);
        write_i32_le(out, 0); // unknown
    }
    write_i32_le(out, bd.used_count);
}

fn write_rect(out: &mut Vec<u8>, r: &Rect) {
    write_f32_le(out, r.y_min);
    write_f32_le(out, r.x_min);
    write_f32_le(out, r.y_max);
    write_f32_le(out, r.x_max);
}

fn write_anchor(out: &mut Vec<u8>, a: &Anchor) {
    write_f32_le(out, a.start_time);
    write_f32_le(out, a.end_time);
    write_f32_le(out, a.first_note_time);
    write_f32_le(out, a.last_note_time);
    write_i8(out, a.fret_id);
    pad(out, 3); // padding
    write_i32_le(out, a.width);
    write_i32_le(out, a.phrase_iteration_id);
}

fn write_anchor_extension(out: &mut Vec<u8>, ae: &AnchorExtension) {
    write_f32_le(out, ae.beat_time);
    write_i8(out, ae.fret_id);
    write_i32_le(out, 0); // unk1
    write_i16_le(out, 0); // unk2
    write_i8(out, 0);     // unk3
}

fn write_finger_print(out: &mut Vec<u8>, fp: &FingerPrint) {
    write_i32_le(out, fp.chord_id);
    write_f32_le(out, fp.start_time);
    write_f32_le(out, fp.end_time);
    write_f32_le(out, fp.first_note_time);
    write_f32_le(out, fp.last_note_time);
}

fn write_note(out: &mut Vec<u8>, n: &Note) {
    write_u32_le(out, n.mask.bits());
    write_u32_le(out, n.flags);
    write_u32_le(out, n.hash);
    write_f32_le(out, n.time);
    write_i8(out, n.string_index);
    write_i8(out, n.fret);
    write_i8(out, n.anchor_fret);
    write_i8(out, n.anchor_width);
    write_i32_le(out, n.chord_id);
    write_i32_le(out, n.chord_notes_id);
    write_i32_le(out, n.phrase_id);
    write_i32_le(out, n.phrase_iteration_id);
    for fp in &n.finger_print_id { write_i16_le(out, *fp); }
    write_i16_le(out, n.next_iter_note);
    write_i16_le(out, n.prev_iter_note);
    write_i16_le(out, n.parent_prev_note);
    write_i8(out, n.slide_to);
    write_i8(out, n.slide_unpitch_to);
    write_i8(out, n.left_hand);
    write_i8(out, n.tap);
    write_i8(out, n.pick_direction);
    write_i8(out, n.slap);
    write_i8(out, n.pluck);
    write_i16_le(out, n.vibrato);
    write_f32_le(out, n.sustain);
    write_f32_le(out, n.max_bend);
    write_count(out, n.bend_data.len());
    for bv in &n.bend_data {
        write_f32_le(out, bv.time);
        write_f32_le(out, bv.step);
        write_i32_le(out, 0); // unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_sng() -> Sng {
        Sng {
            beats: vec![],
            phrases: vec![],
            chords: vec![],
            chord_notes: vec![],
            vocals: vec![],
            symbols_headers: vec![],
            symbols_textures: vec![],
            symbol_definitions: vec![],
            phrase_iterations: vec![],
            phrase_extra_info: vec![],
            new_linked_difficulties: vec![],
            actions: vec![],
            events: vec![],
            tones: vec![],
            dnas: vec![],
            sections: vec![],
            levels: vec![],
            metadata: MetaData {
                max_score: 0.0,
                max_notes_and_chords: 0.0,
                max_notes_and_chords_real: 0.0,
                points_per_note: 0.0,
                first_beat_length: 0.5,
                start_time: 0.0,
                capo_fret_id: -1,
                last_conversion_date_time: "2024-01-01".into(),
                part: 1,
                song_length: 10.0,
                tuning: vec![0, 0, 0, 0, 0, 0],
                first_note_time: 0.0,
                max_difficulty: 0,
            },
        }
    }

    #[test]
    fn roundtrip_minimal_sng() {
        let sng = minimal_sng();
        let bytes = sng_to_bytes(&sng);
        let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
        assert_eq!(parsed.metadata.song_length, sng.metadata.song_length);
        assert_eq!(parsed.metadata.part, sng.metadata.part);
        assert_eq!(parsed.metadata.tuning, sng.metadata.tuning);
        assert_eq!(parsed.beats.len(), 0);
        assert_eq!(parsed.levels.len(), 0);
    }

    #[test]
    fn roundtrip_sng_with_notes() {
        let mut sng = minimal_sng();
        sng.beats.push(Beat {
            time: 0.5,
            measure: 1,
            beat: 1,
            phrase_iteration: 0,
            mask: BeatMask::FIRST_BEAT_OF_MEASURE,
        });
        sng.levels.push(Level {
            difficulty: 0,
            anchors: vec![Anchor {
                start_time: 0.0,
                end_time: 10.0,
                first_note_time: 1.0,
                last_note_time: 2.0,
                fret_id: 5,
                width: 4,
                phrase_iteration_id: 0,
            }],
            anchor_extensions: vec![],
            hand_shapes: vec![],
            arpeggios: vec![],
            notes: vec![Note {
                mask: NoteMask::SINGLE,
                flags: 0,
                hash: 0,
                time: 1.0,
                string_index: 0,
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
                sustain: 0.5,
                max_bend: 0.0,
                bend_data: vec![],
            }],
            average_notes_per_iteration: vec![1.0],
            notes_in_phrase_iterations_excl_ignored: vec![1],
            notes_in_phrase_iterations_all: vec![1],
        });

        let bytes = sng_to_bytes(&sng);
        let parsed = Sng::from_unpacked_bytes(&bytes).unwrap();
        assert_eq!(parsed.beats.len(), 1);
        assert_eq!(parsed.beats[0].time, 0.5);
        assert_eq!(parsed.levels.len(), 1);
        assert_eq!(parsed.levels[0].notes.len(), 1);
        assert_eq!(parsed.levels[0].notes[0].fret, 5);
        assert_eq!(parsed.levels[0].notes[0].time, 1.0);
    }
}
