use rocksmith2014_xml::*;

#[test]
fn test_ebeat_time_conversion() {
    // "1.234" seconds → 1234 ms
    let xml = r#"<?xml version='1.0' encoding='utf-8'?>
<song>
  <title>Test</title>
  <arrangement>Lead</arrangement>
  <part>1</part>
  <offset>0.000</offset>
  <centOffset>0</centOffset>
  <songLength time="10.000" />
  <songNameSort>Test</songNameSort>
  <startBeat>0.000</startBeat>
  <averageTempo>120.000</averageTempo>
  <tuning string0="0" string1="0" string2="0" string3="0" string4="0" string5="0" />
  <capo>0</capo>
  <artistName>Test</artistName>
  <artistNameSort>Test</artistNameSort>
  <albumName>Test</albumName>
  <albumNameSort>Test</albumNameSort>
  <albumYear>2014</albumYear>
  <tonebase>test</tonebase>
  <lastConversionDateTime>2014-01-01</lastConversionDateTime>
  <phrases count="0" />
  <phraseIterations count="0" />
  <newLinkedDiffs count="0" />
  <chordTemplates count="0" />
  <ebeats count="1">
    <ebeat time="1.234" measure="1" />
  </ebeats>
  <sections count="0" />
  <events count="0" />
  <tones>
    <tones count="0" />
  </tones>
  <levels count="0" />
</song>"#;
    let arr = InstrumentalArrangement::from_xml(xml).unwrap();
    assert_eq!(arr.ebeats.len(), 1);
    assert_eq!(arr.ebeats[0].time, 1234);
}

#[test]
fn test_chord_template_defaults() {
    let ct = ChordTemplate::default();
    assert_eq!(ct.frets, [-1i8; 6]);
    assert_eq!(ct.fingers, [-1i8; 6]);
}

#[test]
fn test_note_mask_roundtrip() {
    let mut note = Note::default();
    note.mask |= NoteMask::HAMMER_ON;
    note.time = 1000;
    note.fret = 5;
    note.string = 0;
    note.left_hand = -1;
    note.slide_to = -1;
    note.slide_unpitch_to = -1;

    let mut arr = InstrumentalArrangement::default();
    arr.levels.push(Level {
        difficulty: 0,
        notes: vec![note.clone()],
        ..Default::default()
    });

    let xml = arr.to_xml().unwrap();
    let arr2 = InstrumentalArrangement::from_xml(&xml).unwrap();
    assert_eq!(
        arr2.levels[0].notes[0].mask & NoteMask::HAMMER_ON,
        NoteMask::HAMMER_ON
    );
}

#[test]
fn test_roundtrip_with_notes() {
    let mut arr = InstrumentalArrangement::default();
    arr.meta.arrangement = "Lead".to_string();
    arr.meta.song_name = "Test Song".to_string();
    arr.meta.artist_name = "Test Artist".to_string();
    arr.meta.album_name = "Test Album".to_string();
    arr.meta.song_length = 120000;
    arr.meta.average_tempo = 120.0;

    arr.phrases.push(Phrase {
        name: "phrase1".to_string(),
        max_difficulty: 5,
        ..Default::default()
    });
    arr.phrase_iterations.push(PhraseIteration {
        time: 0,
        end_time: 5000,
        phrase_id: 0,
        hero_levels: None,
    });
    arr.ebeats.push(Ebeat {
        time: 0,
        measure: 1,
    });
    arr.ebeats.push(Ebeat {
        time: 500,
        measure: -1,
    });

    let note = Note {
        time: 1000,
        fret: 7,
        string: 3,
        left_hand: -1,
        slide_to: -1,
        slide_unpitch_to: -1,
        mask: NoteMask::HAMMER_ON,
        ..Default::default()
    };
    arr.levels.push(Level {
        difficulty: 0,
        notes: vec![note],
        ..Default::default()
    });

    let xml = arr.to_xml().unwrap();
    let arr2 = InstrumentalArrangement::from_xml(&xml).unwrap();

    assert_eq!(arr2.meta.arrangement, "Lead");
    assert_eq!(arr2.phrases.len(), 1);
    assert_eq!(arr2.ebeats.len(), 2);
    assert_eq!(arr2.ebeats[0].time, 0);
    assert_eq!(arr2.ebeats[1].measure, -1);
    assert_eq!(arr2.levels[0].notes[0].fret, 7);
    assert_eq!(
        arr2.levels[0].notes[0].mask & NoteMask::HAMMER_ON,
        NoteMask::HAMMER_ON
    );
}

#[test]
fn test_parse_minimal_song() {
    let xml = r#"<?xml version='1.0' encoding='utf-8'?>
<song>
  <title>My Song</title>
  <arrangement>Rhythm</arrangement>
  <part>1</part>
  <offset>-10.000</offset>
  <centOffset>0</centOffset>
  <songLength time="120.000" />
  <songNameSort>My Song</songNameSort>
  <startBeat>0.000</startBeat>
  <averageTempo>120.000</averageTempo>
  <tuning string0="0" string1="0" string2="0" string3="-1" string4="0" string5="0" />
  <capo>0</capo>
  <artistName>Test Artist</artistName>
  <artistNameSort>Test Artist</artistNameSort>
  <albumName>Test Album</albumName>
  <albumNameSort>Test Album</albumNameSort>
  <albumYear>2024</albumYear>
  <tonebase>tone1</tonebase>
  <lastConversionDateTime>2024-01-01</lastConversionDateTime>
  <phrases count="0" />
  <phraseIterations count="0" />
  <newLinkedDiffs count="0" />
  <chordTemplates count="0" />
  <ebeats count="0" />
  <sections count="0" />
  <events count="0" />
  <tones>
    <tones count="0" />
  </tones>
  <levels count="1">
    <level difficulty="0">
      <notes count="1">
        <note time="1.000" sustain="0.500" fret="5" string="0" leftHand="-1"
              slideTo="-1" slideUnpitchTo="-1" tap="0" vibrato="0" maxBend="0"
              hammerOn="1" />
      </notes>
      <chords count="0" />
      <anchors count="1">
        <anchor time="0.000" fret="1" width="4" />
      </anchors>
      <handShapes count="0" />
    </level>
  </levels>
</song>"#;

    let arr = InstrumentalArrangement::from_xml(xml).unwrap();
    assert_eq!(arr.meta.arrangement, "Rhythm");
    assert_eq!(arr.meta.song_name, "My Song");
    assert_eq!(arr.meta.tuning.strings[3], -1);
    assert_eq!(arr.meta.album_year, 2024);
    assert_eq!(arr.levels.len(), 1);
    assert_eq!(arr.levels[0].notes.len(), 1);
    assert_eq!(arr.levels[0].notes[0].fret, 5);
    assert_eq!(arr.levels[0].notes[0].time, 1000);
    assert_eq!(arr.levels[0].notes[0].sustain, 500);
    assert_eq!(arr.levels[0].anchors.len(), 1);
    assert_eq!(arr.levels[0].anchors[0].fret, 1);
    assert!(arr.levels[0].notes[0].mask.contains(NoteMask::HAMMER_ON));
}
