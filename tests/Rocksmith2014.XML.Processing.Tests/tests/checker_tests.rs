use rocksmith2014_xml::{InstrumentalArrangement, Level, MetaData, Note, Phrase, PhraseIteration};
use rocksmith2014_xml_processing::{check_instrumental, Issue, IssueType};

fn basic_arr() -> InstrumentalArrangement {
    InstrumentalArrangement {
        phrases: vec![
            Phrase {
                name: "COUNT".into(),
                ..Default::default()
            },
            Phrase {
                name: "END".into(),
                ..Default::default()
            },
        ],
        phrase_iterations: vec![
            PhraseIteration {
                time: 0,
                phrase_id: 0,
                ..Default::default()
            },
            PhraseIteration {
                time: 5000,
                phrase_id: 1,
                ..Default::default()
            },
        ],
        meta: MetaData {
            song_length: 10_000,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn no_issues_for_valid_arrangement() {
    let arr = basic_arr();
    let issues = check_instrumental(&arr);
    assert!(
        issues.is_empty(),
        "should have no issues, got: {:?}",
        issues
    );
}

#[test]
fn detects_missing_end_phrase() {
    let mut arr = basic_arr();
    arr.phrases = vec![Phrase {
        name: "COUNT".into(),
        ..Default::default()
    }];
    arr.phrase_iterations = vec![PhraseIteration {
        time: 0,
        phrase_id: 0,
        ..Default::default()
    }];
    let issues = check_instrumental(&arr);
    assert!(
        issues
            .iter()
            .any(|i| matches!(i.issue_type(), IssueType::NoEndPhrase)),
        "should detect missing END phrase"
    );
}

#[test]
fn detects_fret_number_more_than_24() {
    let mut arr = basic_arr();
    let notes = vec![Note {
        time: 1000,
        fret: 25,
        ..Default::default()
    }];
    arr.levels = vec![Level {
        notes,
        ..Default::default()
    }];
    let issues = check_instrumental(&arr);
    assert!(
        issues
            .iter()
            .any(|i| matches!(i.issue_type(), IssueType::FretNumberMoreThan24)),
        "should detect fret > 24"
    );
}
