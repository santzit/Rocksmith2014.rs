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
mod phrase_gen;

pub use checker::check_instrumental;
pub use improver::{apply_all_improvements, apply_minimum_improvements};
pub use phrase_gen::generate_phrases;

/// Describes the kind of issue found in an arrangement.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    ApplauseEventWithoutEnd,
    EventBetweenIntroApplause(String),
    NoteLinkedToChord,
    LinkNextMissingTargetNote,
    LinkNextSlideMismatch,
    LinkNextFretMismatch,
    LinkNextBendMismatch,
    IncorrectLinkNext,
    UnpitchedSlideWithLinkNext,
    PhraseChangeOnLinkNextNote,
    DoubleHarmonic,
    SeventhFretHarmonicWithSustain,
    NaturalHarmonicWithBend,
    MissingBendValue,
    OverlappingBendValues,
    ToneChangeOnNote,
    NoteInsideNoguitarSection,
    MissingLinkNextChordNotes,
    FingeringAnchorMismatch,
    PossiblyWrongChordFingering,
    BarreOverOpenStrings,
    MutedStringInNonMutedChord,
    AnchorInsideHandShape,
    AnchorInsideHandShapeAtPhraseBoundary,
    AnchorCloseToUnpitchedSlide,
    FirstPhraseNotEmpty,
    NoEndPhrase,
    MoreThan100Phrases,
    IncorrectMover1Phrase,
    HopoIntoSameNote,
    FingerChangeDuringSlide,
    PositionShiftIntoPullOff,
    InvalidBassArrangementString,
    FretNumberMoreThan24,
    NoteAfterSongEnd,
    TechniqueNoteWithoutSustain,
    LyricWithInvalidChar {
        invalid_char: char,
        custom_font_used: bool,
    },
    LyricTooLong(String),
    LyricsHaveNoLineBreaks,
    InvalidShowlights,
    LowBassTuningWithoutWorkaround,
    IncorrectLowBassTuningForTuningPitch,
}

impl IssueType {
    /// Returns the issue code string.
    pub fn code(&self) -> &'static str {
        match self {
            IssueType::ApplauseEventWithoutEnd => "I01",
            IssueType::EventBetweenIntroApplause(_) => "I02",
            IssueType::NoteLinkedToChord => "I03",
            IssueType::LinkNextMissingTargetNote => "I04",
            IssueType::LinkNextSlideMismatch => "I05",
            IssueType::LinkNextFretMismatch => "I06",
            IssueType::LinkNextBendMismatch => "I07",
            IssueType::IncorrectLinkNext => "I08",
            IssueType::UnpitchedSlideWithLinkNext => "I09",
            IssueType::PhraseChangeOnLinkNextNote => "I10",
            IssueType::DoubleHarmonic => "I11",
            IssueType::SeventhFretHarmonicWithSustain => "I13",
            IssueType::NaturalHarmonicWithBend => "I35",
            IssueType::MissingBendValue => "I14",
            IssueType::OverlappingBendValues => "I34",
            IssueType::ToneChangeOnNote => "I15",
            IssueType::NoteInsideNoguitarSection => "I16",
            IssueType::MissingLinkNextChordNotes => "I18",
            IssueType::FingeringAnchorMismatch => "I20",
            IssueType::PossiblyWrongChordFingering => "I27",
            IssueType::BarreOverOpenStrings => "I28",
            IssueType::MutedStringInNonMutedChord => "I29",
            IssueType::AnchorInsideHandShape => "I21",
            IssueType::AnchorInsideHandShapeAtPhraseBoundary => "I22",
            IssueType::AnchorCloseToUnpitchedSlide => "I23",
            IssueType::FirstPhraseNotEmpty => "I25",
            IssueType::NoEndPhrase => "I26",
            IssueType::MoreThan100Phrases => "I30",
            IssueType::IncorrectMover1Phrase => "I37",
            IssueType::HopoIntoSameNote => "I31",
            IssueType::FingerChangeDuringSlide => "I32",
            IssueType::PositionShiftIntoPullOff => "I33",
            IssueType::InvalidBassArrangementString => "I36",
            IssueType::FretNumberMoreThan24 => "I38",
            IssueType::NoteAfterSongEnd => "I39",
            IssueType::TechniqueNoteWithoutSustain => "I40",
            IssueType::LowBassTuningWithoutWorkaround => "I41",
            IssueType::IncorrectLowBassTuningForTuningPitch => "I42",
            IssueType::LyricWithInvalidChar { .. } => "V01",
            IssueType::LyricTooLong(_) => "V02",
            IssueType::LyricsHaveNoLineBreaks => "V03",
            IssueType::InvalidShowlights => "S01",
        }
    }
}

/// An issue found in an arrangement, optionally associated with a time code.
#[derive(Debug, Clone, PartialEq)]
pub enum Issue {
    General(IssueType),
    WithTimeCode(IssueType, i32),
}

impl Issue {
    /// Returns the issue type.
    pub fn issue_type(&self) -> &IssueType {
        match self {
            Issue::General(t) | Issue::WithTimeCode(t, _) => t,
        }
    }

    /// Returns the time code if present.
    pub fn time_code(&self) -> Option<i32> {
        match self {
            Issue::General(_) => None,
            Issue::WithTimeCode(_, t) => Some(*t),
        }
    }
}
