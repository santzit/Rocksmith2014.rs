use super::bend_value::BendValue;
use super::masks::NoteMask;
use crate::binary_helpers::*;
use std::io::{Read, Write};

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

impl SngRead for Note {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
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
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
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
