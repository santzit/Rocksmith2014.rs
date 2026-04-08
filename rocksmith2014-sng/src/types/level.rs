use super::anchor::Anchor;
use super::anchor_extension::AnchorExtension;
use super::finger_print::FingerPrint;
use super::note::Note;
use crate::binary_helpers::*;
use std::io::{Read, Write};

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
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
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
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
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
