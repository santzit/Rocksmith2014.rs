//! XML entity extension types for Rocksmith 2014.
//!
//! # Creating a sorted entity array
//!
//! ```rust
//! use rocksmith2014_xml_extension::{create_xml_entity_array_from_lists, XmlEntity};
//! use rocksmith2014_xml::{Note, Chord};
//!
//! let note = Note { time: 100, ..Default::default() };
//! let chord = Chord { time: 50, ..Default::default() };
//! let entities = create_xml_entity_array_from_lists(&[note], &[chord]);
//! assert_eq!(entities.len(), 2);
//! // sorted by time: chord at 50 comes before note at 100
//! assert_eq!(entities[0].time_code(), 50);
//! assert_eq!(entities[1].time_code(), 100);
//! ```

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

#[cfg(test)]
mod tests {
    use super::*;
    use rocksmith2014_xml::{Chord, HandShape, Note};

    fn make_note(time: i32) -> Note {
        Note {
            time,
            ..Default::default()
        }
    }

    fn make_chord(time: i32) -> Chord {
        Chord {
            time,
            ..Default::default()
        }
    }

    #[test]
    fn create_from_lists_sorts_by_time() {
        let notes = vec![make_note(100), make_note(200)];
        let chords = vec![make_chord(50), make_chord(150)];
        let entities = create_xml_entity_array_from_lists(&notes, &chords);
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[0].time_code(), 50);
        assert_eq!(entities[1].time_code(), 100);
        assert_eq!(entities[2].time_code(), 150);
        assert_eq!(entities[3].time_code(), 200);
    }

    #[test]
    fn create_from_lists_empty() {
        let entities = create_xml_entity_array_from_lists(&[], &[]);
        assert!(entities.is_empty());
    }

    #[test]
    fn create_from_level_sorts_notes_and_chords() {
        use rocksmith2014_xml::Level;
        let level = Level {
            notes: vec![make_note(300), make_note(100)],
            chords: vec![make_chord(200)],
            ..Default::default()
        };
        let entities = create_xml_entity_array_from_level(&level);
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].time_code(), 100);
        assert_eq!(entities[1].time_code(), 200);
        assert_eq!(entities[2].time_code(), 300);
    }

    #[test]
    fn xml_entity_sustain_note() {
        let note = Note {
            time: 0,
            sustain: 500,
            ..Default::default()
        };
        let entity = XmlEntity::Note(note);
        assert_eq!(entity.sustain(), 500);
    }

    #[test]
    fn xml_entity_sustain_chord_no_chord_notes() {
        let chord = make_chord(0);
        let entity = XmlEntity::Chord(chord);
        assert_eq!(entity.sustain(), 0);
    }

    #[test]
    fn hand_shape_time_returns_start_time() {
        let hs = HandShape {
            start_time: 42,
            ..Default::default()
        };
        assert_eq!(hand_shape_time(&hs), 42);
    }
}
