//! Editor on Fire (EOF) project file writer for Rocksmith 2014.

pub mod helpers;
pub mod types;
mod writer;

pub use types::*;
pub use writer::write_eof_project;
