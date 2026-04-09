use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event as XmlEvent};
use quick_xml::Writer;

use crate::parser::time_to_str;
use crate::types::*;
use crate::Result;

fn write_flag(elem: &mut BytesStart, name: &str, mask: bool) {
    elem.push_attribute((name, if mask { "1" } else { "0" }));
}

fn write_note(writer: &mut Writer<Vec<u8>>, note: &Note) -> Result<()> {
    let mut elem = BytesStart::new("note");
    elem.push_attribute(("time", time_to_str(note.time).as_str()));
    elem.push_attribute(("sustain", time_to_str(note.sustain).as_str()));
    elem.push_attribute(("string", note.string.to_string().as_str()));
    elem.push_attribute(("fret", note.fret.to_string().as_str()));
    elem.push_attribute(("leftHand", note.left_hand.to_string().as_str()));
    elem.push_attribute(("slideTo", note.slide_to.to_string().as_str()));
    elem.push_attribute(("slideUnpitchTo", note.slide_unpitch_to.to_string().as_str()));
    elem.push_attribute(("tap", note.tap.to_string().as_str()));
    elem.push_attribute(("pickDirection", note.pick_direction.to_string().as_str()));
    elem.push_attribute(("slap", note.slap.to_string().as_str()));
    elem.push_attribute(("pluck", note.pluck.to_string().as_str()));
    elem.push_attribute(("vibrato", note.vibrato.to_string().as_str()));
    elem.push_attribute(("maxBend", note.max_bend.to_string().as_str()));
    write_flag(
        &mut elem,
        "linkNext",
        note.mask.contains(NoteMask::LINK_NEXT),
    );
    write_flag(&mut elem, "accent", note.mask.contains(NoteMask::ACCENT));
    write_flag(
        &mut elem,
        "hammerOn",
        note.mask.contains(NoteMask::HAMMER_ON),
    );
    write_flag(
        &mut elem,
        "harmonic",
        note.mask.contains(NoteMask::HARMONIC),
    );
    write_flag(&mut elem, "ignore", note.mask.contains(NoteMask::IGNORE));
    write_flag(
        &mut elem,
        "fretHandMute",
        note.mask.contains(NoteMask::FRET_HAND_MUTE),
    );
    write_flag(
        &mut elem,
        "palmMute",
        note.mask.contains(NoteMask::PALM_MUTE),
    );
    write_flag(&mut elem, "pullOff", note.mask.contains(NoteMask::PULL_OFF));
    write_flag(&mut elem, "tremolo", note.mask.contains(NoteMask::TREMOLO));
    write_flag(
        &mut elem,
        "pinchHarmonic",
        note.mask.contains(NoteMask::PINCH_HARMONIC),
    );
    write_flag(
        &mut elem,
        "rightHand",
        note.mask.contains(NoteMask::RIGHT_HAND),
    );

    if note.bend_values.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut bv_elem = BytesStart::new("bendValues");
        bv_elem.push_attribute(("count", note.bend_values.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(bv_elem))?;
        for bv in &note.bend_values {
            let mut bve = BytesStart::new("bendValue");
            bve.push_attribute(("time", time_to_str(bv.time).as_str()));
            bve.push_attribute(("step", bv.step.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(bve))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("bendValues")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("note")))?;
    }
    Ok(())
}

fn write_chord_note(writer: &mut Writer<Vec<u8>>, cn: &ChordNote) -> Result<()> {
    let mut elem = BytesStart::new("chordNote");
    elem.push_attribute(("string", cn.string.to_string().as_str()));
    elem.push_attribute(("fret", cn.fret.to_string().as_str()));
    elem.push_attribute(("sustain", time_to_str(cn.sustain).as_str()));
    elem.push_attribute(("vibrato", cn.vibrato.to_string().as_str()));
    elem.push_attribute(("slideTo", cn.slide_to.to_string().as_str()));
    elem.push_attribute(("slideUnpitchTo", cn.slide_unpitch_to.to_string().as_str()));
    elem.push_attribute(("leftHand", cn.left_hand.to_string().as_str()));
    write_flag(&mut elem, "linkNext", cn.mask.contains(NoteMask::LINK_NEXT));
    write_flag(&mut elem, "accent", cn.mask.contains(NoteMask::ACCENT));
    write_flag(&mut elem, "hammerOn", cn.mask.contains(NoteMask::HAMMER_ON));
    write_flag(&mut elem, "harmonic", cn.mask.contains(NoteMask::HARMONIC));
    write_flag(&mut elem, "ignore", cn.mask.contains(NoteMask::IGNORE));
    write_flag(
        &mut elem,
        "fretHandMute",
        cn.mask.contains(NoteMask::FRET_HAND_MUTE),
    );
    write_flag(&mut elem, "palmMute", cn.mask.contains(NoteMask::PALM_MUTE));
    write_flag(&mut elem, "pullOff", cn.mask.contains(NoteMask::PULL_OFF));
    write_flag(&mut elem, "tremolo", cn.mask.contains(NoteMask::TREMOLO));
    write_flag(
        &mut elem,
        "pinchHarmonic",
        cn.mask.contains(NoteMask::PINCH_HARMONIC),
    );
    write_flag(
        &mut elem,
        "rightHand",
        cn.mask.contains(NoteMask::RIGHT_HAND),
    );

    if cn.bend_values.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut bv_elem = BytesStart::new("bendValues");
        bv_elem.push_attribute(("count", cn.bend_values.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(bv_elem))?;
        for bv in &cn.bend_values {
            let mut bve = BytesStart::new("bendValue");
            bve.push_attribute(("time", time_to_str(bv.time).as_str()));
            bve.push_attribute(("step", bv.step.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(bve))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("bendValues")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("chordNote")))?;
    }
    Ok(())
}

fn write_chord(writer: &mut Writer<Vec<u8>>, chord: &Chord) -> Result<()> {
    let mut elem = BytesStart::new("chord");
    elem.push_attribute(("time", time_to_str(chord.time).as_str()));
    elem.push_attribute(("chordId", chord.chord_id.to_string().as_str()));
    if chord.sustain > 0 {
        elem.push_attribute(("sustain", time_to_str(chord.sustain).as_str()));
    }
    write_flag(
        &mut elem,
        "fretHandMute",
        chord.mask.contains(ChordMask::FRET_HAND_MUTE),
    );
    write_flag(
        &mut elem,
        "highDensity",
        chord.mask.contains(ChordMask::HIGH_DENSITY),
    );
    write_flag(&mut elem, "hopo", chord.mask.contains(ChordMask::HOPO));
    write_flag(&mut elem, "ignore", chord.mask.contains(ChordMask::IGNORE));
    write_flag(
        &mut elem,
        "linkNext",
        chord.mask.contains(ChordMask::LINK_NEXT),
    );
    write_flag(
        &mut elem,
        "palmMute",
        chord.mask.contains(ChordMask::PALM_MUTE),
    );
    write_flag(&mut elem, "accent", chord.mask.contains(ChordMask::ACCENT));

    if chord.chord_notes.is_empty() {
        writer.write_event(XmlEvent::Empty(elem))?;
    } else {
        writer.write_event(XmlEvent::Start(elem))?;
        let mut cn_elem = BytesStart::new("chordNotes");
        cn_elem.push_attribute(("count", chord.chord_notes.len().to_string().as_str()));
        writer.write_event(XmlEvent::Start(cn_elem))?;
        for cn in &chord.chord_notes {
            write_chord_note(writer, cn)?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("chordNotes")))?;
        writer.write_event(XmlEvent::End(BytesEnd::new("chord")))?;
    }
    Ok(())
}

pub(crate) fn write_text_element(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    content: &str,
) -> Result<()> {
    writer.write_event(XmlEvent::Start(BytesStart::new(tag)))?;
    writer.write_event(XmlEvent::Text(BytesText::new(content)))?;
    writer.write_event(XmlEvent::End(BytesEnd::new(tag)))?;
    Ok(())
}

/// Writes the content of an `InstrumentalArrangement` between the `<song>` and `</song>` tags.
pub(crate) fn write_arrangement(
    arr: &InstrumentalArrangement,
    writer: &mut Writer<Vec<u8>>,
) -> Result<()> {
    write_text_element(writer, "title", &arr.meta.song_name)?;
    write_text_element(writer, "arrangement", &arr.meta.arrangement)?;
    write_text_element(writer, "part", &arr.meta.part.to_string())?;
    write_text_element(
        writer,
        "offset",
        &format!("{:.3}", arr.meta.offset as f64 / 1000.0),
    )?;
    write_text_element(writer, "centOffset", &arr.meta.cent_offset.to_string())?;

    let mut song_length_elem = BytesStart::new("songLength");
    song_length_elem.push_attribute(("time", time_to_str(arr.meta.song_length).as_str()));
    writer.write_event(XmlEvent::Empty(song_length_elem))?;

    write_text_element(
        writer,
        "lastConversionDateTime",
        &arr.meta.last_conversion_date_time,
    )?;
    write_text_element(writer, "startBeat", &time_to_str(arr.meta.start_beat))?;

    let mut avg_tempo_elem = BytesStart::new("averageTempo");
    avg_tempo_elem.push_attribute(("bpm", format!("{:.3}", arr.meta.average_tempo).as_str()));
    writer.write_event(XmlEvent::Empty(avg_tempo_elem))?;

    let mut tuning_elem = BytesStart::new("tuning");
    tuning_elem.push_attribute(("string0", arr.meta.tuning.strings[0].to_string().as_str()));
    tuning_elem.push_attribute(("string1", arr.meta.tuning.strings[1].to_string().as_str()));
    tuning_elem.push_attribute(("string2", arr.meta.tuning.strings[2].to_string().as_str()));
    tuning_elem.push_attribute(("string3", arr.meta.tuning.strings[3].to_string().as_str()));
    tuning_elem.push_attribute(("string4", arr.meta.tuning.strings[4].to_string().as_str()));
    tuning_elem.push_attribute(("string5", arr.meta.tuning.strings[5].to_string().as_str()));
    writer.write_event(XmlEvent::Empty(tuning_elem))?;

    write_text_element(writer, "capo", &arr.meta.capo.to_string())?;
    write_text_element(writer, "artistName", &arr.meta.artist_name)?;
    write_text_element(writer, "artistNameSort", &arr.meta.artist_name_sort)?;
    write_text_element(writer, "albumName", &arr.meta.album_name)?;
    write_text_element(writer, "albumNameSort", &arr.meta.album_name_sort)?;
    write_text_element(writer, "albumYear", &arr.meta.album_year.to_string())?;
    write_text_element(writer, "crowdSpeed", &arr.meta.crowd_speed.to_string())?;

    let ap = &arr.meta.arrangement_properties;
    let mut ap_elem = BytesStart::new("arrangementProperties");
    ap_elem.push_attribute(("represent", ap.represent.to_string().as_str()));
    ap_elem.push_attribute(("bonusArr", ap.bonus_arr.to_string().as_str()));
    ap_elem.push_attribute(("standardTuning", ap.standard_tuning.to_string().as_str()));
    ap_elem.push_attribute((
        "nonStandardChords",
        ap.non_standard_chords.to_string().as_str(),
    ));
    ap_elem.push_attribute(("barrChords", ap.barr_chords.to_string().as_str()));
    ap_elem.push_attribute(("powerChords", ap.power_chords.to_string().as_str()));
    ap_elem.push_attribute(("dropDPower", ap.drop_d_power.to_string().as_str()));
    ap_elem.push_attribute(("openChords", ap.open_chords.to_string().as_str()));
    ap_elem.push_attribute(("fingerPicking", ap.finger_picking.to_string().as_str()));
    ap_elem.push_attribute(("pickDirection", ap.pick_direction.to_string().as_str()));
    ap_elem.push_attribute(("doubleStops", ap.double_stops.to_string().as_str()));
    ap_elem.push_attribute(("palmMutes", ap.palm_mutes.to_string().as_str()));
    ap_elem.push_attribute(("harmonics", ap.harmonics.to_string().as_str()));
    ap_elem.push_attribute(("pinchHarmonics", ap.pinch_harmonics.to_string().as_str()));
    ap_elem.push_attribute(("hopo", ap.hopo.to_string().as_str()));
    ap_elem.push_attribute(("tremolo", ap.tremolo.to_string().as_str()));
    ap_elem.push_attribute(("slides", ap.slides.to_string().as_str()));
    ap_elem.push_attribute(("unpitchedSlides", ap.unpitched_slides.to_string().as_str()));
    ap_elem.push_attribute(("bends", ap.bends.to_string().as_str()));
    ap_elem.push_attribute(("tapping", ap.tapping.to_string().as_str()));
    ap_elem.push_attribute(("vibrato", ap.vibrato.to_string().as_str()));
    ap_elem.push_attribute(("fretHandMutes", ap.fret_hand_mutes.to_string().as_str()));
    ap_elem.push_attribute(("slapPop", ap.slap_pop.to_string().as_str()));
    ap_elem.push_attribute((
        "twoFingerPicking",
        ap.two_finger_picking.to_string().as_str(),
    ));
    ap_elem.push_attribute(("fiveFretChords", ap.five_fret_chords.to_string().as_str()));
    ap_elem.push_attribute(("chordNotes", ap.chord_notes.to_string().as_str()));
    ap_elem.push_attribute(("octaves", ap.octaves.to_string().as_str()));
    ap_elem.push_attribute(("susChords", ap.sus_chords.to_string().as_str()));
    ap_elem.push_attribute((
        "threeFingerChords",
        ap.three_finger_chords.to_string().as_str(),
    ));
    ap_elem.push_attribute(("rhythmSide", ap.rhythm_side.to_string().as_str()));
    ap_elem.push_attribute(("solo", ap.solo.to_string().as_str()));
    ap_elem.push_attribute(("pathLead", ap.path_lead.to_string().as_str()));
    ap_elem.push_attribute(("pathRhythm", ap.path_rhythm.to_string().as_str()));
    ap_elem.push_attribute(("pathBass", ap.path_bass.to_string().as_str()));
    ap_elem.push_attribute(("routingRules", ap.routing_rules.to_string().as_str()));
    writer.write_event(XmlEvent::Empty(ap_elem))?;

    write_text_element(writer, "tonebase", &arr.meta.tone_base)?;
    write_text_element(writer, "tonea", &arr.meta.tone_a)?;
    write_text_element(writer, "toneb", &arr.meta.tone_b)?;
    write_text_element(writer, "tonec", &arr.meta.tone_c)?;
    write_text_element(writer, "toned", &arr.meta.tone_d)?;

    // tones
    let mut tones_elem = BytesStart::new("tones");
    tones_elem.push_attribute(("count", arr.tones.len().to_string().as_str()));
    if arr.tones.is_empty() {
        writer.write_event(XmlEvent::Empty(tones_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(tones_elem))?;
        for tone in &arr.tones {
            let mut te = BytesStart::new("tone");
            te.push_attribute(("time", time_to_str(tone.time).as_str()));
            te.push_attribute(("name", tone.name.as_str()));
            te.push_attribute(("id", tone.id.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(te))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("tones")))?;
    }

    // ebeats
    let mut ebeats_elem = BytesStart::new("ebeats");
    ebeats_elem.push_attribute(("count", arr.ebeats.len().to_string().as_str()));
    if arr.ebeats.is_empty() {
        writer.write_event(XmlEvent::Empty(ebeats_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(ebeats_elem))?;
        for beat in &arr.ebeats {
            let mut be = BytesStart::new("ebeat");
            be.push_attribute(("time", time_to_str(beat.time).as_str()));
            be.push_attribute(("measure", beat.measure.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(be))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("ebeats")))?;
    }

    // phrases
    let mut phrases_elem = BytesStart::new("phrases");
    phrases_elem.push_attribute(("count", arr.phrases.len().to_string().as_str()));
    if arr.phrases.is_empty() {
        writer.write_event(XmlEvent::Empty(phrases_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(phrases_elem))?;
        for phrase in &arr.phrases {
            let mut pe = BytesStart::new("phrase");
            pe.push_attribute(("maxDifficulty", phrase.max_difficulty.to_string().as_str()));
            pe.push_attribute(("name", phrase.name.as_str()));
            pe.push_attribute(("disparity", phrase.disparity.to_string().as_str()));
            pe.push_attribute(("ignore", phrase.ignore.to_string().as_str()));
            pe.push_attribute(("solo", phrase.solo.to_string().as_str()));
            writer.write_event(XmlEvent::Empty(pe))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("phrases")))?;
    }

    // phraseIterations
    let mut pi_elem = BytesStart::new("phraseIterations");
    pi_elem.push_attribute(("count", arr.phrase_iterations.len().to_string().as_str()));
    if arr.phrase_iterations.is_empty() {
        writer.write_event(XmlEvent::Empty(pi_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(pi_elem))?;
        for pi in &arr.phrase_iterations {
            let mut pie = BytesStart::new("phraseIteration");
            pie.push_attribute(("time", time_to_str(pi.time).as_str()));
            pie.push_attribute(("endTime", time_to_str(pi.end_time).as_str()));
            pie.push_attribute(("phraseId", pi.phrase_id.to_string().as_str()));
            pie.push_attribute(("variation", "")); // variation field removed from struct; write empty for XML compatibility
            let hero_levels_nonempty = pi.hero_levels.as_ref().is_some_and(|v| !v.is_empty());
            if !hero_levels_nonempty {
                writer.write_event(XmlEvent::Empty(pie))?;
            } else {
                writer.write_event(XmlEvent::Start(pie))?;
                let hls = pi.hero_levels.as_ref().unwrap();
                let mut hl_elem = BytesStart::new("heroLevels");
                hl_elem.push_attribute(("count", hls.len().to_string().as_str()));
                writer.write_event(XmlEvent::Start(hl_elem))?;
                for hl in hls {
                    let mut hle = BytesStart::new("heroLevel");
                    hle.push_attribute(("hero", hl.hero.to_string().as_str()));
                    hle.push_attribute(("difficulty", hl.difficulty.to_string().as_str()));
                    writer.write_event(XmlEvent::Empty(hle))?;
                }
                writer.write_event(XmlEvent::End(BytesEnd::new("heroLevels")))?;
                writer.write_event(XmlEvent::End(BytesEnd::new("phraseIteration")))?;
            }
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("phraseIterations")))?;
    }

    // linkedDiffs
    let mut ld_elem = BytesStart::new("linkedDiffs");
    ld_elem.push_attribute(("count", arr.linked_diffs.len().to_string().as_str()));
    writer.write_event(XmlEvent::Empty(ld_elem))?;

    // phraseProperties
    let mut pp_elem = BytesStart::new("phraseProperties");
    pp_elem.push_attribute(("count", arr.phrase_properties.len().to_string().as_str()));
    writer.write_event(XmlEvent::Empty(pp_elem))?;

    // chordTemplates
    let mut ct_elem = BytesStart::new("chordTemplates");
    ct_elem.push_attribute(("count", arr.chord_templates.len().to_string().as_str()));
    if arr.chord_templates.is_empty() {
        writer.write_event(XmlEvent::Empty(ct_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(ct_elem))?;
        for ct in &arr.chord_templates {
            let mut cte = BytesStart::new("chordTemplate");
            cte.push_attribute(("chordName", ct.name.as_str()));
            cte.push_attribute(("displayName", ct.display_name.as_str()));
            cte.push_attribute(("finger0", ct.fingers[0].to_string().as_str()));
            cte.push_attribute(("finger1", ct.fingers[1].to_string().as_str()));
            cte.push_attribute(("finger2", ct.fingers[2].to_string().as_str()));
            cte.push_attribute(("finger3", ct.fingers[3].to_string().as_str()));
            cte.push_attribute(("finger4", ct.fingers[4].to_string().as_str()));
            cte.push_attribute(("finger5", ct.fingers[5].to_string().as_str()));
            cte.push_attribute(("fret0", ct.frets[0].to_string().as_str()));
            cte.push_attribute(("fret1", ct.frets[1].to_string().as_str()));
            cte.push_attribute(("fret2", ct.frets[2].to_string().as_str()));
            cte.push_attribute(("fret3", ct.frets[3].to_string().as_str()));
            cte.push_attribute(("fret4", ct.frets[4].to_string().as_str()));
            cte.push_attribute(("fret5", ct.frets[5].to_string().as_str()));
            writer.write_event(XmlEvent::Empty(cte))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("chordTemplates")))?;
    }

    // fretHandMuteTemplates
    let mut fhmt_elem = BytesStart::new("fretHandMuteTemplates");
    fhmt_elem.push_attribute((
        "count",
        arr.fret_hand_mute_templates.len().to_string().as_str(),
    ));
    writer.write_event(XmlEvent::Empty(fhmt_elem))?;

    // controls
    let mut controls_elem = BytesStart::new("controls");
    controls_elem.push_attribute(("count", "0"));
    writer.write_event(XmlEvent::Empty(controls_elem))?;

    // events
    let mut events_elem = BytesStart::new("events");
    events_elem.push_attribute(("count", arr.events.len().to_string().as_str()));
    if arr.events.is_empty() {
        writer.write_event(XmlEvent::Empty(events_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(events_elem))?;
        for ev in &arr.events {
            let mut eve = BytesStart::new("event");
            eve.push_attribute(("time", time_to_str(ev.time).as_str()));
            eve.push_attribute(("code", ev.code.as_str()));
            writer.write_event(XmlEvent::Empty(eve))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("events")))?;
    }

    // sections
    let mut sections_elem = BytesStart::new("sections");
    sections_elem.push_attribute(("count", arr.sections.len().to_string().as_str()));
    if arr.sections.is_empty() {
        writer.write_event(XmlEvent::Empty(sections_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(sections_elem))?;
        for sec in &arr.sections {
            let mut se = BytesStart::new("section");
            se.push_attribute(("name", sec.name.as_str()));
            se.push_attribute(("number", sec.number.to_string().as_str()));
            se.push_attribute(("startTime", time_to_str(sec.start_time).as_str()));
            se.push_attribute(("endTime", time_to_str(sec.end_time).as_str()));
            writer.write_event(XmlEvent::Empty(se))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("sections")))?;
    }

    // levels
    let mut levels_elem = BytesStart::new("levels");
    levels_elem.push_attribute(("count", arr.levels.len().to_string().as_str()));
    if arr.levels.is_empty() {
        writer.write_event(XmlEvent::Empty(levels_elem))?;
    } else {
        writer.write_event(XmlEvent::Start(levels_elem))?;
        for level in &arr.levels {
            let mut le = BytesStart::new("level");
            le.push_attribute(("difficulty", level.difficulty.to_string().as_str()));
            writer.write_event(XmlEvent::Start(le))?;

            // anchors
            let mut anch_elem = BytesStart::new("anchors");
            anch_elem.push_attribute(("count", level.anchors.len().to_string().as_str()));
            if level.anchors.is_empty() {
                writer.write_event(XmlEvent::Empty(anch_elem))?;
            } else {
                writer.write_event(XmlEvent::Start(anch_elem))?;
                for anch in &level.anchors {
                    let mut ae = BytesStart::new("anchor");
                    ae.push_attribute(("time", time_to_str(anch.time).as_str()));
                    ae.push_attribute(("endTime", time_to_str(anch.end_time).as_str()));
                    ae.push_attribute(("fret", anch.fret.to_string().as_str()));
                    ae.push_attribute(("width", anch.width.to_string().as_str()));
                    writer.write_event(XmlEvent::Empty(ae))?;
                }
                writer.write_event(XmlEvent::End(BytesEnd::new("anchors")))?;
            }

            // handShapes
            let mut hs_elem = BytesStart::new("handShapes");
            hs_elem.push_attribute(("count", level.hand_shapes.len().to_string().as_str()));
            if level.hand_shapes.is_empty() {
                writer.write_event(XmlEvent::Empty(hs_elem))?;
            } else {
                writer.write_event(XmlEvent::Start(hs_elem))?;
                for hs in &level.hand_shapes {
                    let mut hse = BytesStart::new("handShape");
                    hse.push_attribute(("chordId", hs.chord_id.to_string().as_str()));
                    hse.push_attribute(("startTime", time_to_str(hs.start_time).as_str()));
                    hse.push_attribute(("endTime", time_to_str(hs.end_time).as_str()));
                    writer.write_event(XmlEvent::Empty(hse))?;
                }
                writer.write_event(XmlEvent::End(BytesEnd::new("handShapes")))?;
            }

            // notes
            let mut notes_elem = BytesStart::new("notes");
            notes_elem.push_attribute(("count", level.notes.len().to_string().as_str()));
            if level.notes.is_empty() {
                writer.write_event(XmlEvent::Empty(notes_elem))?;
            } else {
                writer.write_event(XmlEvent::Start(notes_elem))?;
                for note in &level.notes {
                    write_note(writer, note)?;
                }
                writer.write_event(XmlEvent::End(BytesEnd::new("notes")))?;
            }

            // chords
            let mut chords_elem = BytesStart::new("chords");
            chords_elem.push_attribute(("count", level.chords.len().to_string().as_str()));
            if level.chords.is_empty() {
                writer.write_event(XmlEvent::Empty(chords_elem))?;
            } else {
                writer.write_event(XmlEvent::Start(chords_elem))?;
                for chord in &level.chords {
                    write_chord(writer, chord)?;
                }
                writer.write_event(XmlEvent::End(BytesEnd::new("chords")))?;
            }

            writer.write_event(XmlEvent::End(BytesEnd::new("level")))?;
        }
        writer.write_event(XmlEvent::End(BytesEnd::new("levels")))?;
    }

    Ok(())
}
