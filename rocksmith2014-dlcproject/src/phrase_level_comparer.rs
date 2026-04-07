use crate::arrangement::Arrangement;
use rocksmith2014_sng::Sng;
use std::collections::HashMap;
use uuid::Uuid;

/// Stored phrase difficulty levels keyed by arrangement persistent ID and phrase name.
pub type ProjectLevels = HashMap<Uuid, HashMap<String, i32>>;

/// Converts a NUL-terminated `[u8; 32]` name to a `String`.
fn phrase_name(raw: &[u8; 32]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).into_owned()
}

/// Returns the arrangement IDs whose phrase levels have regressed compared to
/// the stored levels (i.e. the current SNG has fewer levels for a phrase than
/// the stored count).
pub fn compare_levels(stored: &ProjectLevels, arrangements: &[(Arrangement, Sng)]) -> Vec<Uuid> {
    arrangements
        .iter()
        .filter_map(|(arr, sng)| {
            if let Arrangement::Instrumental(inst) = arr {
                let stored_levels = stored.get(&inst.persistent_id)?;

                let has_regression = sng.phrases.iter().any(|phrase| {
                    let name = phrase_name(&phrase.name);
                    stored_levels
                        .get(name.as_str())
                        .map(|&stored_max| stored_max > phrase.max_difficulty)
                        .unwrap_or(false)
                });

                if has_regression {
                    Some(inst.id)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
