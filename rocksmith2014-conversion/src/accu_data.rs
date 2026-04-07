use rocksmith2014_sng::{AnchorExtension, ChordNotes};
use rocksmith2014_xml::{InstrumentalArrangement, PhraseIteration as XmlPhraseIteration};

#[derive(Debug, Default, Clone)]
pub struct NoteCounts {
    pub easy: i32,
    pub medium: i32,
    pub hard: i32,
    pub ignored: i32,
}

/// Accumulated data built up while converting XML notes/chords to SNG notes.
pub struct AccuData {
    /// String masks: [section_index][difficulty (0..36)]
    pub string_masks: Vec<Vec<i8>>,
    /// Created chord notes objects
    pub chord_notes: Vec<ChordNotes>,
    /// Anchor extensions created for slide notes, per difficulty level
    pub anchor_extensions: Vec<Vec<AnchorExtension>>,
    /// Note counts per difficulty per phrase iteration (excluding ignored)
    pub notes_in_phrase_iterations_excl_ignored: Vec<Vec<i32>>,
    /// Note counts per difficulty per phrase iteration (all)
    pub notes_in_phrase_iterations_all: Vec<Vec<i32>>,
    pub note_counts: NoteCounts,
}

impl AccuData {
    pub fn init(arr: &InstrumentalArrangement) -> Self {
        let level_count = arr.levels.len();
        let pi_count = arr.phrase_iterations.len();
        let section_count = arr.sections.len().max(1);
        Self {
            string_masks: vec![vec![0i8; 36]; section_count],
            chord_notes: Vec::new(),
            anchor_extensions: vec![Vec::new(); level_count.max(1)],
            notes_in_phrase_iterations_excl_ignored: vec![vec![0i32; pi_count]; level_count.max(1)],
            notes_in_phrase_iterations_all: vec![vec![0i32; pi_count]; level_count.max(1)],
            note_counts: NoteCounts::default(),
        }
    }

    /// Increments note counts and hero level counts for a new note.
    pub fn add_note(
        &mut self,
        pi_id: usize,
        difficulty: usize,
        pi: &XmlPhraseIteration,
        is_ignored: bool,
    ) {
        if difficulty < self.notes_in_phrase_iterations_all.len()
            && pi_id < self.notes_in_phrase_iterations_all[difficulty].len()
        {
            self.notes_in_phrase_iterations_all[difficulty][pi_id] += 1;
            if !is_ignored {
                self.notes_in_phrase_iterations_excl_ignored[difficulty][pi_id] += 1;
            }
        }

        let (easy, medium, hard) = get_hero_levels(pi);
        let d = difficulty as i32;
        if easy == d {
            self.note_counts.easy += 1;
        }
        if medium == d {
            self.note_counts.medium += 1;
        }
        if hard == d {
            self.note_counts.hard += 1;
            if is_ignored {
                self.note_counts.ignored += 1;
            }
        }
    }

    /// Updates the string mask for a section/difficulty.
    pub fn update_string_mask(&mut self, section_id: usize, difficulty: usize, note_string: usize) {
        if section_id < self.string_masks.len() && difficulty < self.string_masks[section_id].len() {
            self.string_masks[section_id][difficulty] |= 1i8 << note_string;
        }
    }
}

/// Extracts hero level difficulties (easy, medium, hard) from a phrase iteration.
pub fn get_hero_levels(pi: &XmlPhraseIteration) -> (i32, i32, i32) {
    match &pi.hero_levels {
        None => (0, 0, 0),
        Some(hl) if hl.is_empty() => (0, 0, 0),
        Some(hl) => {
            let easy = hl.iter().find(|h| h.hero == 1).map(|h| h.difficulty).unwrap_or(0);
            let medium = hl.iter().find(|h| h.hero == 2).map(|h| h.difficulty).unwrap_or(0);
            let hard = hl.iter().find(|h| h.hero == 3).map(|h| h.difficulty).unwrap_or(0);
            (easy, medium, hard)
        }
    }
}
