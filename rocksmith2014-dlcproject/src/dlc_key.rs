use crate::string_validator;
use rocksmith2014_common::random;

/// Minimum length of a DLC key.
pub const MINIMUM_LENGTH: usize = 5;

fn create_part(s: &str) -> String {
    let p = string_validator::dlc_key(s);
    p.chars().take(5).collect()
}

fn random_chars(count: usize) -> String {
    (0..count).map(|_| random::next_alphabet()).collect()
}

fn create_prefix(charter_name: &str) -> String {
    let name = string_validator::dlc_key(charter_name);
    if name.chars().count() < 2 {
        random_chars(2)
    } else {
        name.chars().take(2).collect()
    }
}

/// Creates a DLC key from the charter name, artist name and title.
pub fn create(charter_name: &str, artist: &str, title: &str) -> String {
    let key = format!(
        "{}{}{}",
        create_prefix(charter_name),
        create_part(artist),
        create_part(title)
    );
    if key.len() < MINIMUM_LENGTH {
        format!("{}{}", key, random_chars(MINIMUM_LENGTH - key.len()))
    } else {
        key
    }
}
