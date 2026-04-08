use std::io::{Read, Write};
use crate::binary_helpers::*;

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
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
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
            max_score, max_notes_and_chords, max_notes_and_chords_real, points_per_note,
            first_beat_length, start_time, capo_fret_id, last_conversion_date_time,
            part, song_length, tuning, first_note_time, max_difficulty,
        })
    }
}

impl SngWrite for MetaData {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
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
