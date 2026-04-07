use std::collections::HashSet;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

static USABLE_CHARS: OnceLock<HashSet<char>> = OnceLock::new();

fn usable_chars() -> &'static HashSet<char> {
    USABLE_CHARS.get_or_init(|| {
        // All characters included in the fonts used in learn-a-song and guitarcade.
        // Matches the SearchValues set in the .NET StringValidator.
        let base: &str = concat!(
            " !\"#$%&'()*+,-./0123456789:;<=>?@",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`",
            "abcdefghijklmnopqrstuvwxyz{|}~",
            // Latin-1 Supplement (¡ through ÿ, skipping soft hyphen U+00AD)
            "¡¢£¤¥¦§¨©ª«¬\u{00AE}¯°±²³´µ¶·¸¹º»¼½¾¿",
            "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞß",
            "àáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ",
            // Latin Extended-A subset used in the game font
            "ĞğİıĲĳŁłŒœŞşŠšŸŽž",
            // Latin Extended-B: ƒ (florin)
            "ƒ",
            // General Punctuation / Currency / Letterlike
            "\u{2013}\u{2014}", // en dash, em dash
            "\u{2018}\u{2019}", // left/right single quotation marks
            "\u{201A}",         // single low-9 quotation mark
            "\u{201C}\u{201D}", // left/right double quotation marks
            "\u{201E}",         // double low-9 quotation mark
            "\u{2020}\u{2021}", // dagger, double dagger
            "\u{2022}",         // bullet
            "\u{2026}",         // horizontal ellipsis
            "\u{2030}",         // per mille sign
            "\u{2039}\u{203A}", // single guillemets
            "\u{2044}",         // fraction slash
            "\u{20AC}",         // euro sign
            "\u{2117}",         // sound recording copyright
            "\u{2122}",         // trade mark sign
            // Geometric shapes / music symbols
            "\u{25A1}\u{25B3}\u{25CB}", // white square, triangle, circle
            "\u{266D}\u{266F}",         // music flat/sharp
        );
        let mut set: HashSet<char> = base.chars().collect();

        // Full-width space
        set.insert('\u{3000}');

        // Japanese CJK punctuation and kana/kanji used in the game font.
        // Rather than enumerating every character, add the full blocks that are
        // present in the .NET usable-characters string.
        // U+3001–U+9FFF  (CJK Unified Ideographs and kana blocks)
        for c in '\u{3001}'..='\u{9FFF}' {
            set.insert(c);
        }
        // Half-width and Full-width Forms (katakana half-width, U+FF61–U+FF9F) + ¥ (U+FFE5)
        for c in '\u{FF61}'..='\u{FF9F}' {
            set.insert(c);
        }
        set.insert('\u{FFE5}');

        set
    })
}

/// Validates a DLC key: only alphanumeric characters allowed.
pub fn dlc_key(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Validates a DLC project field (artist, title, album):
/// only characters included in the fonts the game uses are allowed.
pub fn field(input: &str) -> String {
    let chars = usable_chars();
    input.chars().filter(|c| chars.contains(c)).collect()
}

/// Validates a DLC project sort field: must start with an alphanumeric character.
pub fn sort_field(input: &str) -> String {
    let trimmed = input.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    trimmed.to_string()
}

/// Removes English articles (a, an, the) from the beginning of the input string.
pub fn remove_articles(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.starts_with("the ") {
        input["the ".len()..].to_string()
    } else if lower.starts_with("an ") {
        input["an ".len()..].to_string()
    } else if lower.starts_with("a ") {
        input["a ".len()..].to_string()
    } else {
        input.to_string()
    }
}

/// Removes diacritics from the string using Unicode NFD decomposition.
fn remove_diacritics(input: &str) -> String {
    input
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .nfc()
        .collect()
}

/// Validates a filename without the extension.
/// Removes diacritics, strips characters outside `[^ 0-9a-zA-Z_-]`, and
/// replaces whitespace runs with a single dash.
pub fn file_name(input: &str) -> String {
    let no_diacritics = remove_diacritics(input);

    // Keep only space, alphanumeric, underscore, and hyphen
    let filtered: String = no_diacritics
        .chars()
        .filter(|&c| c == ' ' || c.is_ascii_alphanumeric() || c == '_' || c == '-')
        .collect();

    // Replace runs of spaces with a single dash
    let mut result = String::with_capacity(filtered.len());
    let mut in_space = false;
    for c in filtered.chars() {
        if c == ' ' {
            if !in_space && !result.is_empty() {
                result.push('-');
            }
            in_space = true;
        } else {
            in_space = false;
            result.push(c);
        }
    }
    // Remove any trailing dash introduced by trailing spaces
    result.trim_end_matches('-').to_string()
}
