pub mod beat_writer;
pub mod eof_project_writer;
pub mod event_converter;
pub mod hand_shapes;
pub mod helpers;
pub mod ini_writers;
pub mod note_converter;
pub mod pro_guitar_writer;
pub mod tech_notes;
pub mod tremolo;
pub mod types;
pub mod vocals_writer;
mod write_utils;

pub use eof_project_writer::{
    write_eof_project, EofProTracks, ImportedArrangement, ImportedVocals, Vocal,
};
pub use pro_guitar_writer::prepare_notes;
pub use types::HsResult;
