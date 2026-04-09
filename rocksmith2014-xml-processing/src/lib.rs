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
mod custom_events;
mod double_stop_name_remover;
mod eof_fixes;
mod handshape_adjuster;
mod improver;
mod issue;
mod phrase_gen;
mod phrase_mover;
mod show_lights_checker;
mod vocals_checker;

pub use checker::{
    check_anchors_pub as check_anchors, check_chords_pub as check_chords,
    check_crowd_events_pub as check_crowd_events, check_handshapes_pub as check_handshapes,
    check_instrumental, check_notes_pub as check_notes, check_phrases_pub as check_phrases,
};
pub use custom_events::improve as improve_custom_events;
pub use double_stop_name_remover::improve as improve_double_stop_names;
pub use eof_fixes::{
    fix_chord_notes as eof_fix_chord_notes,
    fix_chord_slide_handshapes as eof_fix_chord_slide_handshapes,
    fix_crowd_events as eof_fix_crowd_events,
    fix_phrase_start_anchors as eof_fix_phrase_start_anchors,
    remove_invalid_chord_note_link_nexts as eof_remove_invalid_chord_note_link_nexts,
};
pub use handshape_adjuster::{lengthen_handshapes, shorten_handshapes};
pub use improver::{
    add_crowd_events, add_ignores, apply_all_improvements, apply_minimum_improvements,
    fix_link_nexts, get_first_note_time, process_chord_names, remove_extra_beats,
    remove_muted_notes_from_chords, remove_overlapping_bend_values, remove_redundant_anchors,
    validate_phrase_names,
};
pub use issue::{Issue, IssueType};
pub use phrase_gen::generate_phrases;
pub use phrase_mover::improve as improve_phrase_mover;
pub use show_lights_checker::check as check_show_lights;
pub use vocals_checker::check as check_vocals;
