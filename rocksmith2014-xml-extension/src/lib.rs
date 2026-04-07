//! XML entity extension types for Rocksmith 2014.

use rocksmith2014_xml::{Chord, HandShape, Level, Note};

/// An XML entity is either a [`Note`] or a [`Chord`].
#[derive(Debug, Clone)]
pub enum XmlEntity {
    Note(Note),
    Chord(Chord),
}

impl XmlEntity {
    /// Returns the time code of the entity.
    pub fn time_code(&self) -> i32 {
        match self {
            XmlEntity::Note(n) => n.time,
            XmlEntity::Chord(c) => c.time,
        }
    }

    /// Returns the sustain of the entity.
    pub fn sustain(&self) -> i32 {
        match self {
            XmlEntity::Note(n) => n.sustain,
            XmlEntity::Chord(c) => c.chord_notes.first().map(|cn| cn.sustain).unwrap_or(0),
        }
    }
}

/// Creates a sorted XML entity array from the notes and chords in the level.
pub fn create_xml_entity_array_from_level(level: &Level) -> Vec<XmlEntity> {
    let mut entities: Vec<XmlEntity> = Vec::with_capacity(level.notes.len() + level.chords.len());
    for note in &level.notes {
        entities.push(XmlEntity::Note(note.clone()));
    }
    for chord in &level.chords {
        entities.push(XmlEntity::Chord(chord.clone()));
    }
    entities.sort_by_key(|e| e.time_code());
    entities
}

/// Creates a sorted XML entity array from separate note and chord slices.
pub fn create_xml_entity_array_from_lists(notes: &[Note], chords: &[Chord]) -> Vec<XmlEntity> {
    let mut entities: Vec<XmlEntity> = Vec::with_capacity(notes.len() + chords.len());
    for note in notes {
        entities.push(XmlEntity::Note(note.clone()));
    }
    for chord in chords {
        entities.push(XmlEntity::Chord(chord.clone()));
    }
    entities.sort_by_key(|e| e.time_code());
    entities
}

/// Returns the start time of a hand shape.
pub fn hand_shape_time(hs: &HandShape) -> i32 {
    hs.start_time
}
