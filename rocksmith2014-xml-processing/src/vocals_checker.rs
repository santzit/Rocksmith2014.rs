use rocksmith2014_xml::{GlyphDefinitions, Vocal};

use crate::{Issue, IssueType};

const MAX_LYRIC_BYTES: usize = 48;

fn is_valid_default_char(c: char) -> bool {
    if c.is_ascii()
        && (c.is_alphanumeric()
            || " !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".contains(c))
    {
        return true;
    }
    matches!(c,
        'À'|'Á'|'Â'|'Ã'|'Ä'|'Å'|'Æ'|'Ç'|'È'|'É'|'Ê'|'Ë'|'Ì'|'Í'|'Î'|'Ï'|
        'Ð'|'Ñ'|'Ò'|'Ó'|'Ô'|'Õ'|'Ö'|'Ø'|'Ù'|'Ú'|'Û'|'Ü'|'Ý'|'Þ'|'ß'|
        'à'|'á'|'â'|'ã'|'ä'|'å'|'æ'|'ç'|'è'|'é'|'ê'|'ë'|'ì'|'í'|'î'|'ï'|
        'ð'|'ñ'|'ò'|'ó'|'ô'|'õ'|'ö'|'ø'|'ù'|'ú'|'û'|'ü'|'ý'|'þ'|'ÿ'
    )
}

/// Checks vocals for issues.
/// Mirrors VocalsChecker.check in the .NET implementation.
pub fn check(font: Option<&GlyphDefinitions>, vocals: &[Vocal]) -> Vec<Issue> {
    if vocals.is_empty() {
        return Vec::new();
    }
    let mut issues = Vec::new();

    let has_line_breaks = vocals
        .iter()
        .any(|v| v.lyric.ends_with('+') || v.lyric.ends_with('-'));
    if !has_line_breaks {
        issues.push(Issue::General(IssueType::LyricsHaveNoLineBreaks));
    }

    for vocal in vocals {
        let lyric = &vocal.lyric;
        if lyric.len() > MAX_LYRIC_BYTES {
            issues.push(Issue::General(IssueType::LyricTooLong(lyric.clone())));
        }
        match font {
            None => {
                for c in lyric.chars() {
                    if c == '+' || c == '-' {
                        continue;
                    }
                    if !is_valid_default_char(c) {
                        issues.push(Issue::General(IssueType::LyricWithInvalidChar {
                            invalid_char: c,
                            custom_font_used: false,
                        }));
                        break;
                    }
                }
            }
            Some(gd) => {
                for c in lyric.chars() {
                    if c == '+' || c == '-' {
                        continue;
                    }
                    let in_font = gd
                        .glyphs
                        .iter()
                        .any(|g| g.symbol.chars().next() == Some(c));
                    if !in_font {
                        issues.push(Issue::General(IssueType::LyricWithInvalidChar {
                            invalid_char: c,
                            custom_font_used: true,
                        }));
                        break;
                    }
                }
            }
        }
    }
    issues
}
