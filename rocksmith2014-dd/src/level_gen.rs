use std::collections::HashMap;

use rocksmith2014_xml::{
    Anchor, Chord, Ebeat, HandShape, HeroLevel, InstrumentalArrangement, Level, Note, Phrase,
    PhraseIteration,
};
use rocksmith2014_xml_extension::create_xml_entity_array_from_lists;

use crate::entity_chooser::{
    choose_anchors, choose_entities, choose_hand_shapes, create_score_map, get_note_count,
    score_entity, NoteScore,
};
use crate::{GeneratorConfig, LevelCountGeneration};

const MIN_LEVELS: usize = 2;
const MAX_LEVELS: usize = 30;

struct PhraseData {
    start: i32,
    end: i32,
    notes: Vec<Note>,
    chords: Vec<Chord>,
    anchors: Vec<Anchor>,
    hand_shapes: Vec<HandShape>,
    beats: Vec<Ebeat>,
    max_chord_strings: usize,
}

fn in_range<T: Clone>(items: &[T], start: i32, end: i32, time_fn: impl Fn(&T) -> i32) -> Vec<T> {
    items
        .iter()
        .filter(|x| {
            let t = time_fn(x);
            t >= start && t < end
        })
        .cloned()
        .collect()
}

fn phrase_data(arr: &InstrumentalArrangement, pi_idx: usize) -> PhraseData {
    let pi = &arr.phrase_iterations[pi_idx];
    let phrase = &arr.phrases[pi.phrase_id as usize];
    let max_d = phrase.max_difficulty as usize;
    let start = pi.time;
    let end = arr
        .phrase_iterations
        .get(pi_idx + 1)
        .map(|x| x.time)
        .unwrap_or(arr.meta.song_length);

    let level = arr
        .levels
        .get(max_d)
        .or_else(|| arr.levels.last())
        .cloned()
        .unwrap_or_default();

    let notes = in_range(&level.notes, start, end, |n| n.time);
    let chords = in_range(&level.chords, start, end, |c| c.time);
    let anchors = in_range(&level.anchors, start, end, |a| a.time);
    let hand_shapes = in_range(&level.hand_shapes, start, end, |h| h.start_time);
    let beats = in_range(&arr.ebeats, start, end, |b| b.time);

    let max_chord_strings = chords
        .iter()
        .filter_map(|c| arr.chord_templates.get(c.chord_id as usize))
        .map(get_note_count)
        .max()
        .unwrap_or(0);

    PhraseData {
        start,
        end,
        notes,
        chords,
        anchors,
        hand_shapes,
        beats,
        max_chord_strings,
    }
}

fn levels_for_phrase(
    config: &GeneratorConfig,
    arr: &InstrumentalArrangement,
    pd: &PhraseData,
) -> Vec<Level> {
    if pd.notes.is_empty() && pd.chords.is_empty() {
        return vec![Level {
            anchors: pd.anchors.clone(),
            ..Default::default()
        }];
    }

    let entities = create_xml_entity_array_from_lists(&pd.notes, &pd.chords);
    let scores: Vec<(i32, NoteScore)> = entities
        .iter()
        .map(|e| {
            let t = e.time_code();
            (t, score_entity(pd.start, pd.end, &pd.beats, t, e))
        })
        .collect();

    let score_map = create_score_map(&scores, entities.len());
    let time_to_score: HashMap<i32, NoteScore> = scores.iter().cloned().collect();
    let mut notes_with_score: HashMap<NoteScore, usize> = HashMap::new();
    for &(_, s) in &scores {
        *notes_with_score.entry(s).or_insert(0) += 1;
    }

    let level_count = match &config.level_count_generation {
        LevelCountGeneration::Constant(n) => *n,
        LevelCountGeneration::Simple | LevelCountGeneration::MlModel => {
            let base = score_map.len();
            let min = MIN_LEVELS.max(pd.max_chord_strings);
            base.clamp(min, MAX_LEVELS)
        }
    };

    (0..level_count)
        .map(|d| {
            if d == level_count - 1 {
                Level {
                    difficulty: d as i8,
                    notes: pd.notes.clone(),
                    chords: pd.chords.clone(),
                    anchors: pd.anchors.clone(),
                    hand_shapes: pd.hand_shapes.clone(),
                }
            } else {
                let dp = (d + 1) as f64 / level_count as f64;
                let (notes, chords) = choose_entities(
                    dp,
                    &score_map,
                    &time_to_score,
                    &notes_with_score,
                    &arr.chord_templates,
                    pd.max_chord_strings,
                    &entities,
                );
                let level_ents: Vec<_> = {
                    let mut v: Vec<_> = notes
                        .iter()
                        .map(|n| rocksmith2014_xml_extension::XmlEntity::Note(n.clone()))
                        .collect();
                    for c in &chords {
                        v.push(rocksmith2014_xml_extension::XmlEntity::Chord(c.clone()));
                    }
                    v.sort_by_key(|e| e.time_code());
                    v
                };
                let anchors = choose_anchors(&level_ents, &pd.anchors, pd.start, pd.end);
                let hand_shapes =
                    choose_hand_shapes(dp, &level_ents, pd.max_chord_strings, &pd.hand_shapes);
                Level {
                    difficulty: d as i8,
                    notes,
                    chords,
                    anchors,
                    hand_shapes,
                }
            }
        })
        .collect()
}

pub fn generate(config: &GeneratorConfig, arr: &mut InstrumentalArrangement) {
    if arr.phrase_iterations.is_empty() || arr.levels.is_empty() {
        return;
    }

    let count = arr.phrase_iterations.len();
    let phrase_data_list: Vec<PhraseData> = (0..count).map(|i| phrase_data(arr, i)).collect();
    let level_data: Vec<Vec<Level>> = phrase_data_list
        .iter()
        .map(|pd| levels_for_phrase(config, arr, pd))
        .collect();

    let level_counts: Vec<usize> = level_data.iter().map(|ld| ld.len()).collect();
    let max_d = *level_counts.iter().max().unwrap_or(&1);

    // Combine levels across phrases
    let mut combined: Vec<Level> = (0..max_d)
        .map(|d| Level {
            difficulty: d as i8,
            ..Default::default()
        })
        .collect();
    for phrase_levels in &level_data {
        for (d, pl) in phrase_levels.iter().enumerate() {
            combined[d].notes.extend_from_slice(&pl.notes);
            combined[d].chords.extend_from_slice(&pl.chords);
            combined[d].anchors.extend_from_slice(&pl.anchors);
            combined[d].hand_shapes.extend_from_slice(&pl.hand_shapes);
        }
    }

    // Build new phrases with updated max_difficulty
    let old_phrases = arr.phrases.clone();
    let old_pis = arr.phrase_iterations.clone();
    let mut new_phrases: Vec<Phrase> = Vec::with_capacity(count);
    let mut new_pis: Vec<PhraseIteration> = Vec::with_capacity(count);

    for (i, old_pi) in old_pis.iter().enumerate() {
        let lc = level_counts[i];
        let max_diff = (lc.saturating_sub(1)) as u8;
        let old_phrase = &old_phrases[old_pi.phrase_id as usize];
        new_phrases.push(Phrase {
            name: old_phrase.name.clone(),
            max_difficulty: max_diff,
            ..Default::default()
        });
        let mut pi = PhraseIteration {
            time: old_pi.time,
            end_time: old_pi.end_time,
            phrase_id: i as u32,
            hero_levels: None,
        };
        if max_diff > 0 {
            let easy = ((max_diff as f64 / 4.0).round() as u8).max(1);
            let medium = ((max_diff as f64 / 2.0).round() as u8).max(1);
            pi.hero_levels = Some(vec![
                HeroLevel {
                    hero: 0,
                    difficulty: easy as i32,
                },
                HeroLevel {
                    hero: 1,
                    difficulty: medium as i32,
                },
                HeroLevel {
                    hero: 2,
                    difficulty: max_diff as i32,
                },
            ]);
        }
        new_pis.push(pi);
    }

    arr.levels = combined;
    arr.phrases = new_phrases;
    arr.phrase_iterations = new_pis;
}
