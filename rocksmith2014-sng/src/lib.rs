//! Rust implementation of the Rocksmith 2014 SNG binary format.
//!
//! SNG files contain the full scored arrangement data (notes, phrases, levels, …)
//! in a compact binary layout. PC and Mac builds use different AES-256-CTR keys to
//! encrypt the payload before it is zlib-compressed.
//!
//! # Reading an encrypted SNG file
//!
//! ```no_run
//! use rocksmith2014_sng::{Sng, Platform};
//!
//! let data = std::fs::read("song_lead.sng").unwrap();
//! let sng = Sng::from_encrypted(&data, Platform::Pc).unwrap();
//!
//! println!("Beats: {}", sng.beats.len());
//! println!("Levels: {}", sng.levels.len());
//! ```
//!
//! # Writing an encrypted SNG file
//!
//! ```no_run
//! use rocksmith2014_sng::{Sng, Platform};
//!
//! let sng = Sng::default();
//! let encrypted = sng.to_encrypted(Platform::Pc).unwrap();
//! std::fs::write("out.sng", &encrypted).unwrap();
//! ```
//!
//! # Round-trip (raw unencrypted bytes)
//!
//! ```
//! use rocksmith2014_sng::Sng;
//!
//! let original = Sng::default();
//! let bytes = original.write().unwrap();
//! let parsed = Sng::read(&bytes).unwrap();
//! assert_eq!(parsed.beats.len(), 0);
//! assert_eq!(parsed.levels.len(), 0);
//! ```

pub(crate) mod binary_helpers;
pub(crate) mod cryptography;
pub mod types;

pub use cryptography::{decrypt_sng, encrypt_sng};
pub use types::*;

/// Writes a zero-terminated UTF-8 string into a fixed-length buffer of `length` bytes.
///
/// Always reserves at least 1 byte for the null terminator, so the maximum
/// number of UTF-8 bytes copied is `length - 1`. The buffer is zero-initialized
/// before copying, so the last byte is always `0`.
///
/// Mirrors `BinaryHelpers.writeZeroTerminatedUTF8String` from Rocksmith2014.NET.
pub fn write_zero_terminated_utf8_string<W: std::io::Write>(
    w: &mut W,
    length: usize,
    s: &str,
) -> std::io::Result<()> {
    let mut arr = vec![0u8; length];
    let utf8 = s.as_bytes();
    let copy_len = utf8.len().min(length - 1);
    arr[..copy_len].copy_from_slice(&utf8[..copy_len]);
    w.write_all(&arr)
}

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
    #[error("Invalid array count: {0}")]
    InvalidArrayCount(i32),
}

pub type Result<T> = std::result::Result<T, Error>;

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
    /// Parses a raw (unencrypted, uncompressed) SNG binary blob.
    pub fn read(data: &[u8]) -> Result<Self> {
        use binary_helpers::*;
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
        use binary_helpers::*;
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

    /// Decrypts and parses a packed (AES-256-CTR encrypted) SNG file.
    pub fn from_encrypted(data: &[u8], platform: Platform) -> Result<Self> {
        let decrypted = decrypt_sng(data, platform)?;
        Sng::read(&decrypted)
    }

    pub fn to_encrypted(&self, platform: Platform) -> Result<Vec<u8>> {
        let raw = self.write()?;
        encrypt_sng(&raw, platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binary_helpers::{SngRead, SngWrite};
    use std::io::Cursor;

    #[test]
    fn test_bend_value_roundtrip() {
        let bv = BendValue {
            time: 1.5,
            step: 0.5,
            unused: 0,
        };
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
