use crate::eof_project_writer::ImportedArrangement;
use crate::hand_shapes::convert_hand_shapes;
use crate::helpers::is_drop_tuning;
use crate::note_converter::convert_notes;
use crate::tech_notes::{combine_tech_notes, get_tech_note_data};
use crate::tremolo::create_tremolo_sections;
use crate::types::{EofNote, EofSection, EofTrackFlag, HsResult, SustainAdjustment};
use crate::write_utils::*;
use rocksmith2014_xml::InstrumentalArrangement;
use std::io::{self, Write};

pub fn write_empty_pro_guitar_track(writer: &mut impl Write, name: &str) -> io::Result<()> {
    write_eof_string(writer, name)?;
    write_u8(writer, 4)?; // format
    write_u8(writer, 5)?; // behaviour
    write_u8(writer, 9)?; // type
    write_i8(writer, -1)?; // difficulty
    write_u32_le(writer, 4)?; // flags
    write_u16_le(writer, 0)?; // compliance flags
    write_u8(writer, 24)?; // highest fret
    let strings: u8 = if name.contains("BASS") { 4 } else { 6 };
    write_u8(writer, strings)?;
    writer.write_all(&vec![0u8; strings as usize])?; // tuning
    write_u32_le(writer, 0)?; // notes
    write_u16_le(writer, 0)?; // sections
    write_u32_le(writer, 0)?; // custom data blocks
    Ok(())
}

fn write_custom_data_block(writer: &mut impl Write, block_id: u32, data: &[u8]) -> io::Result<()> {
    write_i32_le(writer, data.len() as i32 + 4)?;
    write_u32_le(writer, block_id)?;
    writer.write_all(data)
}

fn convert_anchors(capo_fret: i8, inst: &InstrumentalArrangement) -> Vec<EofSection> {
    let mut result = Vec::new();
    for (diff, level) in inst.levels.iter().enumerate() {
        for anchor in &level.anchors {
            let fret = if capo_fret > 0 && anchor.fret > 0 {
                anchor.fret - capo_fret
            } else {
                anchor.fret
            };
            result.push(EofSection::create(
                diff as u8,
                anchor.time as u32,
                fret as u32,
                0,
            ));
        }
    }
    result
}

fn convert_tones(inst: &InstrumentalArrangement) -> Vec<EofSection> {
    inst.tones
        .iter()
        .map(|t| {
            let end_time = if t.name == inst.meta.tone_base { 1 } else { 0 };
            EofSection {
                name: t.name.clone(),
                ..EofSection::create(255, t.time as u32, end_time, 0)
            }
        })
        .collect()
}

fn get_arrangement_type(inst: &InstrumentalArrangement) -> u8 {
    match inst.meta.arrangement.to_lowercase().as_str() {
        "combo" => 1,
        "rhythm" => 2,
        "lead" => 3,
        "bass" => 4,
        _ => 0,
    }
}

pub fn prepare_notes(
    hand_shape_result: &[HsResult],
    inst: &InstrumentalArrangement,
    mut notes: Vec<EofNote>,
) -> Vec<EofNote> {
    use std::collections::HashMap;
    let updates: HashMap<(u8, u32), u32> = hand_shape_result
        .iter()
        .flat_map(|r| match r {
            HsResult::AdjustSustains(s) => s
                .iter()
                .map(|x: &SustainAdjustment| ((x.difficulty, x.time), x.new_sustain))
                .collect::<Vec<_>>(),
            _ => vec![],
        })
        .collect();

    if !updates.is_empty() {
        for n in &mut notes {
            if let Some(&new_len) = updates.get(&(n.difficulty, n.position)) {
                n.length = new_len;
            }
        }
    }

    if inst.meta.capo > 0 {
        let capo = inst.meta.capo as u8;
        for n in &mut notes {
            let new_frets: Vec<u8> = n
                .frets
                .iter()
                .map(|&f| if f == 0 { 0 } else { f - capo })
                .collect();
            n.frets = new_frets;
            n.slide_end_fret = n.slide_end_fret.map(|f| f - capo);
            n.unpitched_slide_end_fret = n.unpitched_slide_end_fret.map(|f| f - capo);
        }
    }

    notes
}

pub fn write_pro_track(
    writer: &mut impl Write,
    name: &str,
    imported: &ImportedArrangement,
) -> io::Result<()> {
    let inst = &imported.data;
    let (notes, fingering_data, tech_notes_vecs) = convert_notes(inst);
    let tones = convert_tones(inst);
    let anchors = convert_anchors(inst.meta.capo, inst);
    let hand_shape_result = convert_hand_shapes(inst, &notes);
    let notes = prepare_notes(&hand_shape_result, inst, notes);
    let hand_shapes: Vec<EofSection> = hand_shape_result
        .iter()
        .filter_map(|r| match r {
            HsResult::SectionCreated(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    let fingering_data: Vec<u8> = fingering_data.into_iter().flatten().collect();
    let all_tech_notes: Vec<EofNote> = tech_notes_vecs.into_iter().flatten().collect();
    let tech_notes = combine_tech_notes(all_tech_notes);
    let tech_notes_data = get_tech_note_data(&tech_notes);
    let tremolo_sections = create_tremolo_sections(&notes);

    let section_count: u16 = [
        &hand_shapes[..],
        &tremolo_sections[..],
        &anchors[..],
        &tones[..],
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .count() as u16;

    let ap = &inst.meta.arrangement_properties;
    let mut track_flag = EofTrackFlag::UNLIMITED_DIFFS | EofTrackFlag::ALT_NAME;
    if ap.bass_pick != 0 {
        track_flag |= EofTrackFlag::RS_PICKED_BASS;
    }
    if ap.bonus_arr != 0 {
        track_flag |= EofTrackFlag::RS_BONUS_ARR;
    } else if ap.represent == 0 {
        track_flag |= EofTrackFlag::RS_ALT_ARR;
    }

    let string_count: u8 = if ap.path_bass != 0 {
        let more_than_four = inst
            .levels
            .iter()
            .flat_map(|l| l.notes.iter())
            .any(|n| n.string > 3);
        if more_than_four {
            6
        } else {
            4
        }
    } else {
        6
    };

    let tuning: Vec<u8> = inst.meta.tuning.strings[..string_count as usize]
        .iter()
        .map(|&s| s as u8)
        .collect();

    let track_type: u8 = if name.contains("BONUS") {
        14
    } else if name.contains("BASS") {
        8
    } else {
        9
    };

    let tuning_not_honored: u8 = if is_drop_tuning(&inst.meta.tuning.strings) {
        0
    } else {
        1
    };

    let custom_data_count: u32 = 2
        + if !fingering_data.is_empty() { 1 } else { 0 }
        + if inst.meta.capo > 0 { 1 } else { 0 }
        + if !tech_notes_data.is_empty() { 1 } else { 0 }
        + if inst.levels.len() > 5 { 1 } else { 0 };

    // Write track header
    write_eof_string(writer, name)?;
    write_u8(writer, 4)?; // format
    write_u8(writer, 5)?; // behaviour
    write_u8(writer, track_type)?;
    write_i8(writer, -1)?;
    write_u32_le(writer, track_flag.bits())?;
    write_u16_le(writer, 0)?;

    // Alternative name
    write_eof_string(writer, &imported.custom_name)?;

    write_u8(writer, 24)?; // highest fret
    write_u8(writer, string_count)?;
    writer.write_all(&tuning)?;

    // Notes
    write_notes(writer, &notes)?;

    // Sections
    write_u16_le(writer, section_count)?;
    if !hand_shapes.is_empty() {
        write_u16_le(writer, 10)?;
        write_sections(writer, &hand_shapes)?;
    }
    if !tremolo_sections.is_empty() {
        write_u16_le(writer, 14)?;
        write_sections(writer, &tremolo_sections)?;
    }
    if !anchors.is_empty() {
        write_u16_le(writer, 16)?;
        write_sections(writer, &anchors)?;
    }
    if !tones.is_empty() {
        write_u16_le(writer, 18)?;
        write_sections(writer, &tones)?;
    }

    // Custom data blocks
    write_u32_le(writer, custom_data_count)?;

    if !fingering_data.is_empty() {
        write_custom_data_block(writer, 2, &fingering_data)?;
    }
    write_custom_data_block(writer, 3, &[get_arrangement_type(inst)])?;
    write_custom_data_block(writer, 4, &[tuning_not_honored])?;
    if inst.meta.capo > 0 {
        write_custom_data_block(writer, 6, &[inst.meta.capo as u8])?;
    }
    if !tech_notes_data.is_empty() {
        write_custom_data_block(writer, 7, &tech_notes_data)?;
    }
    if inst.levels.len() > 5 {
        write_custom_data_block(writer, 9, &[inst.levels.len() as u8])?;
    }

    Ok(())
}
