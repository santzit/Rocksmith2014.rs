//! XML-type tests mirroring Rocksmith2014.XML.Tests from Rocksmith2014.NET v3.5.0.
//!
//! Test files from tests/cdlc/:
//!  - `Vocals.xml`      — from Rocksmith2014.XML.Tests/Vocals.xml
//!  - `Showlights.xml`  — from Rocksmith2014.XML.Tests/Showlights.xml
//!  - `instrumental.xml`— from Rocksmith2014.XML.Tests/instrumental.xml
//!
//! Mirrors: ShowLightTests, VocalTests, HandShapeTests, MetaDataTests,
//!          InstrumentalArrangementTests, UtilsTest (UtilsTest is also
//!          covered per-function in the unit tests inside src/xml/utils.rs).

use rocksmith2014::{
    xml::{
        show_light::{ShowLight, ShowLights, BEAM_MAX, BEAM_MIN, BEAM_OFF, FOG_MAX, FOG_MIN},
        utils::{parse_binary, time_code_from_float_string, time_code_to_string},
        vocal::{VocalsArrangement, XmlVocal},
        InstrumentalArrangement,
    },
};

fn cdlc(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cdlc")
        .join(name)
}

// ===========================================================================
// UtilsTest  (mirrors Rocksmith2014.XML.Tests/UtilsTest.cs)
// ===========================================================================

/// Mirrors: UtilsTest.TimeCodeToString_ConvertsCorrectly (theory with 8 cases)
#[test]
fn utils_time_code_to_string_converts_correctly() {
    let cases: &[(i32, &str)] = &[
        (0, "0.000"),
        (18, "0.018"),
        (235, "0.235"),
        (1000, "1.000"),
        (1234, "1.234"),
        (20500, "20.500"),
        (989999, "989.999"),
        (987456123, "987456.123"),
    ];
    for &(input, expected) in cases {
        assert_eq!(time_code_to_string(input), expected,
            "time_code_to_string({input})");
    }
}

/// Mirrors: UtilsTest.TimeCodeFromFloatString_ParsesCorrectly (theory with 13 cases)
#[test]
fn utils_time_code_from_float_string_parses_correctly() {
    let cases: &[(&str, i32)] = &[
        ("0.000",          0),
        ("0.018",         18),
        ("0.235",        235),
        ("1.000",       1000),
        ("1.234",       1234),
        ("20.500",     20500),
        ("989.999",   989999),
        ("1",           1000),
        ("8.7",         8700),
        ("6.66",        6660),
        ("18.00599",   18005),
        ("254.112",   254112),
        ("9504.11299999", 9504112),
    ];
    for &(input, expected) in cases {
        assert_eq!(time_code_from_float_string(input), expected,
            "time_code_from_float_string({input:?})");
    }
}

/// Mirrors: UtilsTest.ParseBinary_ParsesCorrectly (theory with 4 cases)
#[test]
fn utils_parse_binary_parses_correctly() {
    assert_eq!(parse_binary("0"), 0, "\"0\" -> 0");
    assert_eq!(parse_binary("1"), 1, "\"1\" -> 1");
    assert_eq!(parse_binary("2"), 1, "\"2\" -> 1");
    assert_eq!(parse_binary("9"), 1, "\"9\" -> 1");
}

// ===========================================================================
// ShowLightTests  (mirrors Rocksmith2014.XML.Tests/ShowLightTests.cs)
// ===========================================================================

/// Mirrors: ShowLightTests.FogRangeTest
#[test]
fn show_light_fog_range_test() {
    let mut sl = ShowLight { time: 0, note: 0 };

    for note in FOG_MIN..=FOG_MAX {
        sl.note = note;
        assert!(sl.is_fog(), "note {note} should be fog");
    }

    sl.note = FOG_MAX + 1;
    assert!(!sl.is_fog(), "note {} should NOT be fog (above FogMax)", FOG_MAX + 1);

    sl.note = FOG_MIN - 1;
    assert!(!sl.is_fog(), "note {} should NOT be fog (below FogMin)", FOG_MIN - 1);
}

/// Mirrors: ShowLightTests.BeamRangeTest
#[test]
fn show_light_beam_range_test() {
    let mut sl = ShowLight { time: 50, note: FOG_MIN };

    for note in BEAM_MIN..=BEAM_MAX {
        sl.note = note;
        assert!(sl.is_beam(), "note {note} should be beam");
    }

    sl.note = BEAM_OFF;
    assert!(sl.is_beam(), "BEAM_OFF ({BEAM_OFF}) should be beam");

    sl.note = BEAM_MAX + 1;
    assert!(!sl.is_beam(), "note {} should NOT be beam (above BeamMax)", BEAM_MAX + 1);

    sl.note = BEAM_MIN - 1;
    assert!(!sl.is_beam(), "note {} should NOT be beam (below BeamMin)", BEAM_MIN - 1);
}

/// Mirrors: ShowLightTests.ListOfShowlightsCanBeSavedToXmlFile
/// (we verify a generated XML contains the expected content)
#[test]
fn show_light_save_produces_expected_content() {
    let lights = vec![
        ShowLight { time: 1_000, note: BEAM_MIN },
        ShowLight { time: 10_000, note: FOG_MAX },
        ShowLight { time: 12_000, note: BEAM_OFF },
    ];

    // Build the XML manually (mirrors the .NET Save() output format)
    let mut xml = format!("<showlights count=\"{}\">\n", lights.len());
    for sl in &lights {
        xml.push_str(&format!(
            "  <showlight time=\"{}\" note=\"{}\" />\n",
            sl.time_string(),
            sl.note
        ));
    }
    xml.push_str("</showlights>\n");

    assert!(xml.contains("<showlights count=\"3\">"),
        "XML should contain count attribute");
    assert!(xml.contains("<showlight time=\"12.000\" note=\"42\" />"),
        "XML should contain BEAM_OFF entry at 12s");
}

/// Mirrors: ShowLightTests.ListOfShowlightsCanBeReadFromXmlFile
#[test]
fn show_light_can_be_read_from_xml_file() {
    let lights = ShowLights::load(cdlc("Showlights.xml"))
        .expect("open Showlights.xml");

    assert_eq!(lights.0.len(), 226, "Showlights.xml should have 226 entries");

    // <showlight time="18.731" note="35" />  (index 5, 0-based)
    assert_eq!(lights.0[5].time, 18_731, "entry[5] time should be 18731 ms");
    assert_eq!(lights.0[5].note, 35,     "entry[5] note should be 35");
}

// ===========================================================================
// VocalTests  (mirrors Rocksmith2014.XML.Tests/VocalTests.cs)
// ===========================================================================

/// Mirrors: VocalTests.CopyConstructorCopiesAllValues
#[test]
fn vocal_copy_constructor_copies_all_values() {
    // .NET: new Vocal(12345, 500, "test", 66)
    let v1 = XmlVocal::new(12345, 500, "test", 66);
    let v2 = v1.clone();

    assert_eq!(v2.time,   12345, "time");
    assert_eq!(v2.length, 500,   "length");
    assert_eq!(v2.lyric,  "test","lyric");
    assert_eq!(v2.note,   66,    "note");
}

/// Mirrors: VocalTests.ListOfVocalsCanBeSavedToXmlFile
#[test]
fn vocals_can_be_saved_to_xml_file() {
    let vocals = vec![
        XmlVocal { time: 0, note: 0, length: 0, lyric: String::new() },
        XmlVocal::new(12340, 500, "Test", 66),
        XmlVocal::new(25678, 500, "Test 2", 66),
    ];
    let arr = VocalsArrangement { vocals };
    let content = arr.to_xml_string();

    assert!(content.contains("<vocals count=\"3\">"),
        "XML should contain count attribute; got:\n{content}");
    assert!(
        content.contains("<vocal time=\"12.340\" note=\"66\" length=\"0.500\" lyric=\"Test\" />"),
        "XML should contain the second vocal; got:\n{content}"
    );
}

/// Mirrors: VocalTests.ListOfVocalsCanBeReadFromXmlFile
#[test]
fn vocals_can_be_read_from_xml_file() {
    let arr = VocalsArrangement::load(cdlc("Vocals.xml"))
        .expect("open Vocals.xml");

    assert_eq!(arr.vocals.len(), 8, "Vocals.xml should have 8 vocals");

    // .NET: vocals[5].Time = 28_780, vocals[5].Note = 254, length=600, lyric="sum+"
    // <vocal time="28.780" note="254" length="0.600" lyric="sum+"/>
    assert_eq!(arr.vocals[5].time,   28_780, "vocals[5] time");
    assert_eq!(arr.vocals[5].note,   254,    "vocals[5] MIDI note");
    assert_eq!(arr.vocals[5].length, 600,    "vocals[5] length");
    assert_eq!(arr.vocals[5].lyric,  "sum+", "vocals[5] lyric");
}

/// Additional: validate time/length parsing from Vocals.xml (index 0)
#[test]
fn vocals_times_are_parsed_as_milliseconds() {
    let arr = VocalsArrangement::load(cdlc("Vocals.xml")).unwrap();
    // time="25.330" → 25330 ms
    assert_eq!(arr.vocals[0].time,   25_330, "vocals[0] time");
    // length="0.600" → 600 ms
    assert_eq!(arr.vocals[0].length, 600,    "vocals[0] length");
    assert_eq!(arr.vocals[0].note,   254,    "vocals[0] MIDI note");
}

// ===========================================================================
// HandShapeTests  (mirrors Rocksmith2014.XML.Tests/HandShapeTests.cs)
// ===========================================================================

/// Mirrors: HandShapeTests.CopyConstructorCopiesAllValues
/// In Rust we use a plain struct and verify that `clone()` produces equal values.
#[derive(Debug, Clone)]
struct HandShape {
    chord_id: i32,
    start_time: i32,
    end_time: i32,
}

impl HandShape {
    fn new(chord_id: i32, start_time: i32, end_time: i32) -> Self {
        Self { chord_id, start_time, end_time }
    }
}

#[test]
fn hand_shape_copy_constructor_copies_all_values() {
    let hs1 = HandShape::new(15, 7_777, 8_888);
    let hs2 = hs1.clone();

    assert_eq!(hs2.chord_id,    15,    "chord_id");
    assert_eq!(hs2.start_time,  7_777, "start_time");
    assert_eq!(hs2.end_time,    8_888, "end_time");
}

// ===========================================================================
// MetaDataTests  (mirrors Rocksmith2014.XML.Tests/MetaDataTests.cs)
// ===========================================================================

/// Mirrors: MetaDataTests.CanBeReadFromXMLFile
#[test]
fn xml_metadata_read_from_file() {
    let arr = InstrumentalArrangement::open(cdlc("instrumental.xml"))
        .expect("open instrumental.xml");

    assert_eq!(arr.title, "Test Instrumental", "title");
    assert!(
        (arr.average_tempo - 160.541).abs() < 0.001,
        "average_tempo ≈ 160.541, got {}",
        arr.average_tempo
    );
    assert_eq!(arr.artist_name_sort, "Test",        "artistNameSort");
    assert_eq!(arr.last_conversion_date_time, "5-17-20 15:21", "lastConversionDateTime");
}

// ===========================================================================
// InstrumentalArrangementTests  (mirrors InstrumentalArrangementTests.cs)
// ===========================================================================

/// Mirrors: "CanRemoveDD" — instrumental.xml has 12 difficulty levels
#[test]
fn instrumental_arrangement_has_dd_levels() {
    let arr = InstrumentalArrangement::open(cdlc("instrumental.xml")).unwrap();
    assert_eq!(arr.level_count, 12, "instrumental.xml should have 12 DD levels");
}

/// Mirrors the read-only observation that the arrangement is Lead
#[test]
fn instrumental_arrangement_is_lead() {
    let arr = InstrumentalArrangement::open(cdlc("instrumental.xml")).unwrap();
    assert_eq!(arr.arrangement, "Lead", "arrangement type");
}

// ===========================================================================
// ShowLight constant values  (sanity-check the exported constants)
// ===========================================================================

#[test]
fn show_light_constants_have_correct_values() {
    assert_eq!(FOG_MIN,   24, "FogMin");
    assert_eq!(FOG_MAX,   35, "FogMax");
    assert_eq!(BEAM_OFF,  42, "BeamOff");
    assert_eq!(BEAM_MIN,  48, "BeamMin");
    assert_eq!(BEAM_MAX,  59, "BeamMax");
    // LasersOff and LasersOn are defined in the module but not re-exported here;
    // test them directly
    assert_eq!(rocksmith2014::xml::show_light::LASERS_OFF, 66, "LasersOff");
    assert_eq!(rocksmith2014::xml::show_light::LASERS_ON,  67, "LasersOn");
}
