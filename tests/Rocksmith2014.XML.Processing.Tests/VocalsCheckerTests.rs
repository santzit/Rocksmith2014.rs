use rocksmith2014_xml::{GlyphDefinition, GlyphDefinitions, Vocal};
use rocksmith2014_xml_processing::vocals_checker::check as check_vocals;
use rocksmith2014_xml_processing::issue::IssueType;

#[test]
fn no_issues_for_empty_vocals() {
    let vocals: Vec<Vocal> = vec![];
    let result = check_vocals(None, &vocals);
    assert!(result.is_empty());
}

#[test]
fn detects_character_not_in_default_font() {
    let vocals = vec![
        Vocal { time: 0, length: 50, lyric: "Test+".into(), note: 0 },
        Vocal { time: 100, length: 50, lyric: "Nopeあ".into(), note: 0 },
    ];
    let result = check_vocals(None, &vocals);
    assert!(!result.is_empty());
    assert!(result.iter().any(|i| matches!(i.issue_type(), IssueType::LyricWithInvalidChar { invalid_char, .. } if *invalid_char == 'あ')));
}

#[test]
fn accepts_characters_in_default_font() {
    let vocals = vec![
        Vocal { time: 0, length: 50, lyric: "Test+".into(), note: 0 },
        Vocal { time: 100, length: 50, lyric: "ÄöÖÅå-".into(), note: 0 },
        Vocal { time: 200, length: 50, lyric: "àè+?&#\"".into(), note: 0 },
    ];
    let result = check_vocals(None, &vocals);
    assert!(result.is_empty());
}

#[test]
fn detects_lyric_that_is_too_long_ascii() {
    let lyric: String = "A".repeat(49);
    let vocals = vec![
        Vocal { time: 0, length: 10, lyric: "Test+".into(), note: 0 },
        Vocal { time: 0, length: 50, lyric: lyric.clone(), note: 0 },
    ];
    let result = check_vocals(None, &vocals);
    assert!(result.iter().any(|i| matches!(i.issue_type(), IssueType::LyricTooLong(l) if l == &lyric)));
}

#[test]
fn detects_lyrics_without_line_breaks() {
    let vocals = vec![
        Vocal { time: 0, length: 50, lyric: "Line".into(), note: 0 },
        Vocal { time: 50, length: 100, lyric: "Test".into(), note: 0 },
    ];
    let result = check_vocals(None, &vocals);
    assert!(result.iter().any(|i| matches!(i.issue_type(), IssueType::LyricsHaveNoLineBreaks)));
}

#[test]
fn ignores_special_characters_when_using_custom_font() {
    let vocals = vec![
        Vocal { time: 0, length: 100, lyric: "あ+".into(), note: 0 },
        Vocal { time: 50, length: 50, lyric: "あ-".into(), note: 0 },
        Vocal { time: 80, length: 50, lyric: "あ+".into(), note: 0 },
    ];
    let custom_font = GlyphDefinitions {
        glyphs: vec![GlyphDefinition { symbol: "あ".into(), ..Default::default() }],
        ..Default::default()
    };
    let result = check_vocals(Some(&custom_font), &vocals);
    assert!(result.is_empty());
}
