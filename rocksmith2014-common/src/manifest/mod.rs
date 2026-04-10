//! Manifest types for Rocksmith 2014 DLC.
//!
//! Mirrors `Rocksmith2014.Common/Manifest/` from the .NET implementation.

pub mod arrangement_properties;
pub mod attributes;
pub mod chord_template;
#[allow(clippy::module_inception)]
pub mod manifest;
pub mod phrase;
pub mod phrase_iteration;
pub mod section;
pub mod tone;
pub mod tuning;

pub use arrangement_properties::ArrangementProperties;
pub use attributes::{Attributes, AttributesContainer};
pub use chord_template::ChordTemplate;
pub use manifest::Manifest;
pub use phrase::Phrase;
pub use phrase_iteration::PhraseIteration;
pub use section::Section;
pub use tone::{Gear, Pedal, Tone, ToneError};
pub use tuning::Tuning;

// Re-export ToneDescriptor from the parent crate module
pub use crate::tone_descriptor::{combine_ui_names, get_descriptions_or_default, ToneDescriptor};
