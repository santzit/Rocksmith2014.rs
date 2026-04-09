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

pub use checker::{
    check_instrumental, check_notes_pub as check_notes, check_chords_pub as check_chords,
    check_handshapes_pub as check_handshapes, check_anchors_pub as check_anchors,
    check_phrases_pub as check_phrases, check_crowd_events_pub as check_crowd_events,
};
pub use improver::{
    apply_all_improvements, apply_minimum_improvements,
    validate_phrase_names, add_ignores, fix_link_nexts,
    remove_overlapping_bend_values, add_crowd_events, remove_extra_beats,
    remove_redundant_anchors, process_chord_names,
};
pub use issue::{Issue, IssueType};
pub use phrase_gen::generate_phrases;
