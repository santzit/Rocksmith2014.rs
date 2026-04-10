use rocksmith2014_xml::{ChordTemplate, InstrumentalArrangement};
use rocksmith2014_xml_processing::improvers::improver::process_chord_names;
use rocksmith2014_xml_processing::improvers::double_stop_name_remover::improve as improve_double_stop_names;

fn template(name: &str) -> ChordTemplate {
    ChordTemplate {
        name: name.into(),
        display_name: name.into(),
        fingers: [-1; 6],
        frets: [-1; 6],
    }
}

fn template_with_frets(name: &str, frets: [i8; 6]) -> ChordTemplate {
    ChordTemplate {
        name: name.into(),
        display_name: name.into(),
        fingers: [-1; 6],
        frets,
    }
}

#[test]
fn fixes_minor_chord_names() {
    let c1 = template("Emin");
    let c2 = template("Amin7");
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c1, c2],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert_eq!(arr.chord_templates[0].name, "Em");
    assert_eq!(arr.chord_templates[0].display_name, "Em");
    assert!(!arr.chord_templates.iter().any(|c| c.name.contains("min") || c.display_name.contains("min")));
}

#[test]
fn fixes_arp_chord_names() {
    let c = template("E-arp");
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert_eq!(arr.chord_templates[0].name, "E");
    assert_eq!(arr.chord_templates[0].display_name, "E-arp");
}

#[test]
fn fixes_nop_chord_names() {
    let c = template("CMaj7-nop");
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert_eq!(arr.chord_templates[0].name, "CMaj7");
    assert_eq!(arr.chord_templates[0].display_name, "CMaj7-nop");
}

#[test]
fn can_convert_chords_to_arpeggios() {
    let c = template("CminCONV");
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert_eq!(arr.chord_templates[0].name, "Cm");
    assert_eq!(arr.chord_templates[0].display_name, "Cm-arp");
}

#[test]
fn fixes_empty_chord_names() {
    let c = template(" ");
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c],
        ..Default::default()
    };
    process_chord_names(&mut arr);
    assert_eq!(arr.chord_templates[0].name.len(), 0);
    assert_eq!(arr.chord_templates[0].display_name.len(), 0);
}

#[test]
fn removes_name_from_double_stops() {
    // Two-string chord (double stop), not a power chord
    let c1 = template_with_frets("E-5", [0, 1, -1, -1, -1, -1]);
    let c2 = template_with_frets("E3", [-1, 7, 6, -1, -1, -1]);
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c1, c2],
        ..Default::default()
    };
    let standard_tuning = [0i16; 6];
    improve_double_stop_names(&standard_tuning, &mut arr);
    assert_eq!(arr.chord_templates[0].name, "");
    assert_eq!(arr.chord_templates[1].name, "");
}

#[test]
fn does_not_remove_name_from_full_chord() {
    let c1 = template_with_frets("Em", [0, 2, 2, 0, 0, 0]);
    let mut arr = InstrumentalArrangement {
        chord_templates: vec![c1],
        ..Default::default()
    };
    let standard_tuning = [0i16; 6];
    improve_double_stop_names(&standard_tuning, &mut arr);
    assert_eq!(arr.chord_templates[0].name, "Em");
}
