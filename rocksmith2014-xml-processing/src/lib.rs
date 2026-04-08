//! XML arrangement processing: checking and improving Rocksmith 2014 arrangements.
//!
//! # Checking a minimal valid arrangement
//!
//! ```rust
//! use rocksmith2014_xml::{InstrumentalArrangement, MetaData, Phrase, PhraseIteration};
//! use rocksmith2014_xml_processing::check_instrumental;
//!
//! let arr = InstrumentalArrangement {
//!     phrases: vec![
//!         Phrase { name: "COUNT".into(), ..Default::default() },
//!         Phrase { name: "END".into(), ..Default::default() },
//!     ],
//!     phrase_iterations: vec![
//!         PhraseIteration { time: 0, phrase_id: 0, ..Default::default() },
//!         PhraseIteration { time: 5000, phrase_id: 1, ..Default::default() },
//!     ],
//!     meta: MetaData { song_length: 10_000, ..Default::default() },
//!     ..Default::default()
//! };
//! let issues = check_instrumental(&arr);
//! assert!(issues.is_empty());
//! ```

mod checker;
mod improver;
mod issue;
mod phrase_gen;

pub use checker::check_instrumental;
pub use improver::{apply_all_improvements, apply_minimum_improvements};
pub use issue::{Issue, IssueType};
pub use phrase_gen::generate_phrases;
