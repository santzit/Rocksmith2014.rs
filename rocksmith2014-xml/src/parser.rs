use quick_xml::events::{BytesStart, Event as XmlEvent};
use quick_xml::Reader;

use crate::types::*;
use crate::Result;

/// Mirrors .NET `Utils.TimeCodeFromFloatString`: truncates (does not round)
/// the fractional part to at most 3 decimal digits, then returns the combined
/// integer millisecond value.
pub(crate) fn time_from_str(s: &str) -> i32 {
    match s.find('.') {
        None => s.parse::<i32>().unwrap_or(0) * 1000,
        Some(dot) => {
            let whole: i32 = s[..dot].parse().unwrap_or(0);
            // Take at most 3 characters after the dot, pad with '0' on the right
            let frac_src = &s[dot + 1..];
            let mut frac_chars = [b'0'; 3];
            for (i, b) in frac_src.bytes().take(3).enumerate() {
                frac_chars[i] = b;
            }
            let frac: i32 = std::str::from_utf8(&frac_chars)
                .ok()
                .and_then(|f| f.parse().ok())
                .unwrap_or(0);
            whole * 1000 + frac
        }
    }
}

pub(crate) fn time_to_str(ms: i32) -> String {
    format!("{:.3}", ms as f64 / 1000.0)
}

fn get_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name)
        .and_then(|a| String::from_utf8(a.value.to_vec()).ok())
}

fn flag_from_attr(e: &BytesStart, name: &[u8]) -> bool {
    get_attr(e, name).as_deref() == Some("1")
}

fn parse_note_mask(e: &BytesStart) -> NoteMask {
    let mut mask = NoteMask::empty();
    if flag_from_attr(e, b"linkNext") {
        mask |= NoteMask::LINK_NEXT;
    }
    if flag_from_attr(e, b"accent") {
        mask |= NoteMask::ACCENT;
    }
    if flag_from_attr(e, b"hammerOn") {
        mask |= NoteMask::HAMMER_ON;
    }
    if flag_from_attr(e, b"harmonic") {
        mask |= NoteMask::HARMONIC;
    }
    if flag_from_attr(e, b"ignore") {
        mask |= NoteMask::IGNORE;
    }
    if flag_from_attr(e, b"fretHandMute") {
        mask |= NoteMask::FRET_HAND_MUTE;
    }
    if flag_from_attr(e, b"palmMute") {
        mask |= NoteMask::PALM_MUTE;
    }
    if flag_from_attr(e, b"pullOff") {
        mask |= NoteMask::PULL_OFF;
    }
    if flag_from_attr(e, b"tremolo") {
        mask |= NoteMask::TREMOLO;
    }
    if flag_from_attr(e, b"pinchHarmonic") {
        mask |= NoteMask::PINCH_HARMONIC;
    }
    if flag_from_attr(e, b"pickDirection") {
        mask |= NoteMask::PICK_DIRECTION;
    }
    if flag_from_attr(e, b"slap") {
        mask |= NoteMask::SLAP;
    }
    if flag_from_attr(e, b"pluck") {
        mask |= NoteMask::PLUCK;
    }
    if flag_from_attr(e, b"rightHand") {
        mask |= NoteMask::RIGHT_HAND;
    }
    mask
}

fn parse_chord_mask(e: &BytesStart) -> ChordMask {
    let mut mask = ChordMask::empty();
    if flag_from_attr(e, b"fretHandMute") {
        mask |= ChordMask::FRET_HAND_MUTE;
    }
    if flag_from_attr(e, b"highDensity") {
        mask |= ChordMask::HIGH_DENSITY;
    }
    if flag_from_attr(e, b"hopo") {
        mask |= ChordMask::HOPO;
    }
    if flag_from_attr(e, b"ignore") {
        mask |= ChordMask::IGNORE;
    }
    if flag_from_attr(e, b"linkNext") {
        mask |= ChordMask::LINK_NEXT;
    }
    if flag_from_attr(e, b"palmMute") {
        mask |= ChordMask::PALM_MUTE;
    }
    if flag_from_attr(e, b"accent") {
        mask |= ChordMask::ACCENT;
    }
    mask
}

fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String> {
    loop {
        match reader.read_event()? {
            XmlEvent::Text(t) => return Ok(t.unescape()?.into_owned()),
            XmlEvent::End(_) => return Ok(String::new()),
            XmlEvent::Eof => return Ok(String::new()),
            _ => {}
        }
    }
}

fn parse_bend_values(reader: &mut Reader<&[u8]>) -> Result<Vec<BendValue>> {
    let mut values = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"bendValue" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let step = get_attr(&e, b"step")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                values.push(BendValue {
                    time,
                    step,
                    unk5: 0,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"bendValue" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let step = get_attr(&e, b"step")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                values.push(BendValue {
                    time,
                    step,
                    unk5: 0,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"bendValues" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(values)
}

fn parse_note(reader: &mut Reader<&[u8]>, e: &BytesStart, is_start: bool) -> Result<Note> {
    let time = get_attr(e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
    let string = get_attr(e, b"string")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let fret = get_attr(e, b"fret")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let sustain = get_attr(e, b"sustain")
        .map(|s| time_from_str(&s))
        .unwrap_or(0);
    let vibrato = get_attr(e, b"vibrato")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let slide_to = get_attr(e, b"slideTo")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let slide_unpitch_to = get_attr(e, b"slideUnpitchTo")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let left_hand = get_attr(e, b"leftHand")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let tap = get_attr(e, b"tap")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let pick_direction = get_attr(e, b"pickDirection")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let slap = get_attr(e, b"slap")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let pluck = get_attr(e, b"pluck")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let max_bend = get_attr(e, b"maxBend")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let mask = parse_note_mask(e);

    let mut bend_values = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"bendValues" => {
                    bend_values = parse_bend_values(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"bendValues" => {
                    // empty bend values
                }
                XmlEvent::End(ce) if ce.name().as_ref() == b"note" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(Note {
        time,
        string,
        fret,
        sustain,
        vibrato,
        slide_to,
        slide_unpitch_to,
        left_hand,
        tap,
        pick_direction,
        slap,
        pluck,
        max_bend,
        mask,
        bend_values,
    })
}

fn parse_chord_note(
    reader: &mut Reader<&[u8]>,
    e: &BytesStart,
    is_start: bool,
) -> Result<ChordNote> {
    let string = get_attr(e, b"string")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let fret = get_attr(e, b"fret")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let sustain = get_attr(e, b"sustain")
        .map(|s| time_from_str(&s))
        .unwrap_or(0);
    let vibrato = get_attr(e, b"vibrato")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let slide_to = get_attr(e, b"slideTo")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let slide_unpitch_to = get_attr(e, b"slideUnpitchTo")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let left_hand = get_attr(e, b"leftHand")
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    let mask = parse_note_mask(e);

    let mut bend_values = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"bendValues" => {
                    bend_values = parse_bend_values(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"bendValues" => {}
                XmlEvent::End(ce) if ce.name().as_ref() == b"chordNote" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(ChordNote {
        string,
        fret,
        sustain,
        vibrato,
        slide_to,
        slide_unpitch_to,
        left_hand,
        bend_values,
        mask,
    })
}

fn parse_chord_notes_list(reader: &mut Reader<&[u8]>) -> Result<Vec<ChordNote>> {
    let mut notes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chordNote" => {
                notes.push(parse_chord_note(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chordNote" => {
                notes.push(parse_chord_note(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"chordNotes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(notes)
}

fn parse_chord(reader: &mut Reader<&[u8]>, e: &BytesStart, is_start: bool) -> Result<Chord> {
    let time = get_attr(e, b"time").map(|s| time_from_str(&s)).unwrap_or(0);
    let chord_id = get_attr(e, b"chordId")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let sustain = get_attr(e, b"sustain")
        .map(|s| time_from_str(&s))
        .unwrap_or(0);
    let mask = parse_chord_mask(e);

    let mut chord_notes = vec![];

    if is_start {
        loop {
            match reader.read_event()? {
                XmlEvent::Start(ce) if ce.name().as_ref() == b"chordNotes" => {
                    chord_notes = parse_chord_notes_list(reader)?;
                }
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"chordNotes" => {}
                // Direct <chordNote> children (some XML variants omit the wrapper)
                XmlEvent::Empty(ce) if ce.name().as_ref() == b"chordNote" => {
                    chord_notes.push(parse_chord_note(reader, &ce, false)?);
                }
                XmlEvent::Start(ce) if ce.name().as_ref() == b"chordNote" => {
                    chord_notes.push(parse_chord_note(reader, &ce, true)?);
                }
                XmlEvent::End(ce) if ce.name().as_ref() == b"chord" => break,
                XmlEvent::Eof => break,
                _ => {}
            }
        }
    }

    Ok(Chord {
        time,
        chord_id,
        sustain,
        mask,
        chord_notes,
    })
}

fn parse_notes(reader: &mut Reader<&[u8]>) -> Result<Vec<Note>> {
    let mut notes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"note" => {
                notes.push(parse_note(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"note" => {
                notes.push(parse_note(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"notes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(notes)
}

fn parse_chords(reader: &mut Reader<&[u8]>) -> Result<Vec<Chord>> {
    let mut chords = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chord" => {
                chords.push(parse_chord(reader, &e, false)?);
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chord" => {
                chords.push(parse_chord(reader, &e, true)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"chords" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(chords)
}

fn parse_anchors(reader: &mut Reader<&[u8]>) -> Result<Vec<Anchor>> {
    let mut anchors = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"anchor" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let fret = get_attr(&e, b"fret")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let width = get_attr(&e, b"width")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4);
                anchors.push(Anchor {
                    time,
                    end_time,
                    fret,
                    width,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"anchor" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let fret = get_attr(&e, b"fret")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let width = get_attr(&e, b"width")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4);
                anchors.push(Anchor {
                    time,
                    end_time,
                    fret,
                    width,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"anchors" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(anchors)
}

fn parse_hand_shapes(reader: &mut Reader<&[u8]>) -> Result<Vec<HandShape>> {
    let mut shapes = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"handShape" => {
                let chord_id = get_attr(&e, b"chordId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let start_time = get_attr(&e, b"startTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                shapes.push(HandShape {
                    chord_id,
                    start_time,
                    end_time,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"handShape" => {
                let chord_id = get_attr(&e, b"chordId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let start_time = get_attr(&e, b"startTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                shapes.push(HandShape {
                    chord_id,
                    start_time,
                    end_time,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"handShapes" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(shapes)
}

fn parse_level(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Level> {
    let difficulty = get_attr(e, b"difficulty")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut anchors = vec![];
    let mut hand_shapes = vec![];
    let mut notes = vec![];
    let mut chords = vec![];

    loop {
        match reader.read_event()? {
            XmlEvent::Start(ce) => match ce.name().as_ref() {
                b"anchors" => {
                    anchors = parse_anchors(reader)?;
                }
                b"handShapes" => {
                    hand_shapes = parse_hand_shapes(reader)?;
                }
                b"notes" => {
                    notes = parse_notes(reader)?;
                }
                b"chords" => {
                    chords = parse_chords(reader)?;
                }
                _ => {
                    let end = ce.to_end().into_owned();
                    reader.read_to_end(end.name())?;
                }
            },
            XmlEvent::Empty(ce) => match ce.name().as_ref() {
                b"anchors" => {}
                b"handShapes" => {}
                b"notes" => {}
                b"chords" => {}
                _ => {}
            },
            XmlEvent::End(ce) if ce.name().as_ref() == b"level" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }

    Ok(Level {
        difficulty,
        anchors,
        hand_shapes,
        notes,
        chords,
    })
}

fn parse_levels(reader: &mut Reader<&[u8]>) -> Result<Vec<Level>> {
    let mut levels = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Start(e) if e.name().as_ref() == b"level" => {
                levels.push(parse_level(reader, &e)?);
            }
            XmlEvent::End(e) if e.name().as_ref() == b"levels" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(levels)
}

fn parse_ebeats(reader: &mut Reader<&[u8]>) -> Result<Vec<Ebeat>> {
    let mut beats = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"ebeat" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let measure = get_attr(&e, b"measure")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1);
                beats.push(Ebeat { time, measure });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"ebeat" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let measure = get_attr(&e, b"measure")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1);
                beats.push(Ebeat { time, measure });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"ebeats" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(beats)
}

fn parse_phrases(reader: &mut Reader<&[u8]>) -> Result<Vec<Phrase>> {
    let mut phrases = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phrase" => {
                let max_difficulty = get_attr(&e, b"maxDifficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let disparity = get_attr(&e, b"disparity")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let ignore = get_attr(&e, b"ignore")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let solo = get_attr(&e, b"solo")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                phrases.push(Phrase {
                    max_difficulty,
                    name,
                    disparity,
                    ignore,
                    solo,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phrase" => {
                let max_difficulty = get_attr(&e, b"maxDifficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let disparity = get_attr(&e, b"disparity")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let ignore = get_attr(&e, b"ignore")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let solo = get_attr(&e, b"solo")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                phrases.push(Phrase {
                    max_difficulty,
                    name,
                    disparity,
                    ignore,
                    solo,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phrases" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(phrases)
}

fn parse_hero_levels(reader: &mut Reader<&[u8]>) -> Result<Vec<HeroLevel>> {
    let mut hero_levels = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"heroLevel" => {
                let hero = get_attr(&e, b"hero")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                hero_levels.push(HeroLevel { hero, difficulty });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"heroLevel" => {
                let hero = get_attr(&e, b"hero")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                hero_levels.push(HeroLevel { hero, difficulty });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"heroLevels" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(hero_levels)
}

fn parse_phrase_iterations(reader: &mut Reader<&[u8]>) -> Result<Vec<PhraseIteration>> {
    let mut iterations = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phraseIteration" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let phrase_id = get_attr(&e, b"phraseId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                iterations.push(PhraseIteration {
                    time,
                    end_time,
                    phrase_id,
                    hero_levels: None,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phraseIteration" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let phrase_id = get_attr(&e, b"phraseId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let mut hero_levels = None;
                loop {
                    match reader.read_event()? {
                        XmlEvent::Start(ce) if ce.name().as_ref() == b"heroLevels" => {
                            hero_levels = Some(parse_hero_levels(reader)?);
                        }
                        XmlEvent::Empty(ce) if ce.name().as_ref() == b"heroLevels" => {}
                        XmlEvent::End(ce) if ce.name().as_ref() == b"phraseIteration" => break,
                        XmlEvent::Eof => break,
                        _ => {}
                    }
                }
                iterations.push(PhraseIteration {
                    time,
                    end_time,
                    phrase_id,
                    hero_levels,
                });
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phraseIterations" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(iterations)
}

fn parse_chord_templates(reader: &mut Reader<&[u8]>, end_tag: &[u8]) -> Result<Vec<ChordTemplate>> {
    let mut templates = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"chordTemplate" => {
                templates.push(parse_chord_template_from_elem(&e));
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"chordTemplate" => {
                templates.push(parse_chord_template_from_elem(&e));
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == end_tag => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(templates)
}

fn parse_chord_template_from_elem(e: &BytesStart) -> ChordTemplate {
    ChordTemplate {
        name: get_attr(e, b"chordName").unwrap_or_default(),
        display_name: get_attr(e, b"displayName").unwrap_or_default(),
        fingers: [
            get_attr(e, b"finger0")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"finger1")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"finger2")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"finger3")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"finger4")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"finger5")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
        ],
        frets: [
            get_attr(e, b"fret0")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"fret1")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"fret2")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"fret3")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"fret4")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            get_attr(e, b"fret5")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
        ],
    }
}

fn parse_events(reader: &mut Reader<&[u8]>) -> Result<Vec<ArrangementEvent>> {
    let mut events = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"event" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let code = get_attr(&e, b"code").unwrap_or_default();
                events.push(ArrangementEvent { time, code });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"event" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let code = get_attr(&e, b"code").unwrap_or_default();
                events.push(ArrangementEvent { time, code });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"events" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(events)
}

fn parse_sections(reader: &mut Reader<&[u8]>) -> Result<Vec<Section>> {
    let mut sections = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"section" => {
                let name = get_attr(&e, b"name").unwrap_or_default();
                let number = get_attr(&e, b"number")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let start_time = get_attr(&e, b"startTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                sections.push(Section {
                    name,
                    number,
                    start_time,
                    end_time,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"section" => {
                let name = get_attr(&e, b"name").unwrap_or_default();
                let number = get_attr(&e, b"number")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let start_time = get_attr(&e, b"startTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let end_time = get_attr(&e, b"endTime")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                sections.push(Section {
                    name,
                    number,
                    start_time,
                    end_time,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"sections" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(sections)
}

fn parse_tones(reader: &mut Reader<&[u8]>) -> Result<Vec<ToneChange>> {
    let mut tones = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"tone" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let id = get_attr(&e, b"id")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                tones.push(ToneChange { time, name, id });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"tone" => {
                let time = get_attr(&e, b"time")
                    .map(|s| time_from_str(&s))
                    .unwrap_or(0);
                let name = get_attr(&e, b"name").unwrap_or_default();
                let id = get_attr(&e, b"id")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                tones.push(ToneChange { time, name, id });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"tones" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(tones)
}

fn parse_linked_diffs(reader: &mut Reader<&[u8]>) -> Result<Vec<LinkedDiff>> {
    let mut diffs = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"linkedDiff" => {
                let parent_id = get_attr(&e, b"parentId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let child_id = get_attr(&e, b"childId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                diffs.push(LinkedDiff {
                    parent_id,
                    child_id,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"linkedDiff" => {
                let parent_id = get_attr(&e, b"parentId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let child_id = get_attr(&e, b"childId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                diffs.push(LinkedDiff {
                    parent_id,
                    child_id,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"linkedDiffs" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(diffs)
}

fn parse_phrase_properties(reader: &mut Reader<&[u8]>) -> Result<Vec<PhraseProperty>> {
    let mut props = vec![];
    loop {
        match reader.read_event()? {
            XmlEvent::Empty(e) if e.name().as_ref() == b"phraseProperty" => {
                let phrase_id = get_attr(&e, b"phraseId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let redundant = get_attr(&e, b"redundant")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let level_jump = get_attr(&e, b"levelJump")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let empty = get_attr(&e, b"empty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                props.push(PhraseProperty {
                    phrase_id,
                    redundant,
                    level_jump,
                    empty,
                    difficulty,
                });
            }
            XmlEvent::Start(e) if e.name().as_ref() == b"phraseProperty" => {
                let phrase_id = get_attr(&e, b"phraseId")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let redundant = get_attr(&e, b"redundant")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let level_jump = get_attr(&e, b"levelJump")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let empty = get_attr(&e, b"empty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let difficulty = get_attr(&e, b"difficulty")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                props.push(PhraseProperty {
                    phrase_id,
                    redundant,
                    level_jump,
                    empty,
                    difficulty,
                });
                let end = e.to_end().into_owned();
                reader.read_to_end(end.name())?;
            }
            XmlEvent::End(e) if e.name().as_ref() == b"phraseProperties" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(props)
}

fn parse_arrangement_properties(e: &BytesStart) -> ArrangementProperties {
    ArrangementProperties {
        represent: get_attr(e, b"represent")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bonus_arr: get_attr(e, b"bonusArr")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        standard_tuning: get_attr(e, b"standardTuning")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1),
        non_standard_chords: get_attr(e, b"nonStandardChords")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        barr_chords: get_attr(e, b"barreChords")
            .or_else(|| get_attr(e, b"barrChords"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        power_chords: get_attr(e, b"powerChords")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        drop_d_power: get_attr(e, b"dropDPower")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        open_chords: get_attr(e, b"openChords")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        finger_picking: get_attr(e, b"fingerPicking")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        pick_direction: get_attr(e, b"pickDirection")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        double_stops: get_attr(e, b"doubleStops")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        palm_mutes: get_attr(e, b"palmMutes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        harmonics: get_attr(e, b"harmonics")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        pinch_harmonics: get_attr(e, b"pinchHarmonics")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        hopo: get_attr(e, b"hopo")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        tremolo: get_attr(e, b"tremolo")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        slides: get_attr(e, b"slides")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        unpitched_slides: get_attr(e, b"unpitchedSlides")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bends: get_attr(e, b"bends")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        tapping: get_attr(e, b"tapping")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        vibrato: get_attr(e, b"vibrato")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        fret_hand_mutes: get_attr(e, b"fretHandMutes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        slap_pop: get_attr(e, b"slapPop")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        two_finger_picking: get_attr(e, b"twoFingerPicking")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        five_fret_chords: get_attr(e, b"fiveFretChords")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        chord_notes: get_attr(e, b"syncopation")
            .or_else(|| get_attr(e, b"chordNotes"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        octaves: get_attr(e, b"octaves")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        sus_chords: get_attr(e, b"sustain")
            .or_else(|| get_attr(e, b"susChords"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        three_finger_chords: get_attr(e, b"threeFingerChords")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        rhythm_side: get_attr(e, b"rhythmSide")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        solo: get_attr(e, b"solo")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        path_lead: get_attr(e, b"pathLead")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        path_rhythm: get_attr(e, b"pathRhythm")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        path_bass: get_attr(e, b"pathBass")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        routing_rules: get_attr(e, b"routingRules")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bass_pick: get_attr(e, b"bassPick")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        synth_lead: get_attr(e, b"synthLead")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        synth_bass: get_attr(e, b"synthBass")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    }
}

fn parse_tuning(e: &BytesStart) -> Tuning {
    Tuning {
        strings: [
            get_attr(e, b"string0")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            get_attr(e, b"string1")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            get_attr(e, b"string2")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            get_attr(e, b"string3")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            get_attr(e, b"string4")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            get_attr(e, b"string5")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        ],
    }
}

pub(crate) fn parse_song(reader: &mut Reader<&[u8]>) -> Result<InstrumentalArrangement> {
    let mut arr = InstrumentalArrangement::default();

    loop {
        match reader.read_event()? {
            XmlEvent::Start(e) => {
                match e.name().as_ref() {
                    b"title" => {
                        arr.meta.song_name = read_text_content(reader)?;
                    }
                    b"arrangement" => {
                        arr.meta.arrangement = read_text_content(reader)?;
                    }
                    b"part" => {
                        let s = read_text_content(reader)?;
                        arr.meta.part = s.parse().unwrap_or(1);
                    }
                    b"offset" => {
                        let s = read_text_content(reader)?;
                        arr.meta.offset = time_from_str(&s);
                    }
                    b"centOffset" => {
                        let s = read_text_content(reader)?;
                        arr.meta.cent_offset = s.parse().unwrap_or(0.0);
                    }
                    b"songLength" => {
                        let s = read_text_content(reader)?;
                        arr.meta.song_length = time_from_str(&s);
                    }
                    b"averageTempo" => {
                        let s = read_text_content(reader)?;
                        arr.meta.average_tempo = s.parse().unwrap_or(120.0);
                    }
                    b"lastConversionDateTime" => {
                        arr.meta.last_conversion_date_time = read_text_content(reader)?;
                    }
                    b"startBeat" => {
                        let s = read_text_content(reader)?;
                        arr.meta.start_beat = time_from_str(&s);
                    }
                    b"capo" => {
                        let s = read_text_content(reader)?;
                        arr.meta.capo = s.parse().unwrap_or(0);
                    }
                    b"artistName" => {
                        arr.meta.artist_name = read_text_content(reader)?;
                    }
                    b"artistNameSort" => {
                        arr.meta.artist_name_sort = read_text_content(reader)?;
                    }
                    b"albumName" => {
                        arr.meta.album_name = read_text_content(reader)?;
                    }
                    b"albumNameSort" => {
                        arr.meta.album_name_sort = read_text_content(reader)?;
                    }
                    b"albumYear" => {
                        let s = read_text_content(reader)?;
                        arr.meta.album_year = s.parse().unwrap_or(0);
                    }
                    b"crowdSpeed" => {
                        let s = read_text_content(reader)?;
                        arr.meta.crowd_speed = s.parse().unwrap_or(1);
                    }
                    b"tonebase" => {
                        arr.meta.tone_base = read_text_content(reader)?;
                    }
                    b"tonea" => {
                        arr.meta.tone_a = read_text_content(reader)?;
                    }
                    b"toneb" => {
                        arr.meta.tone_b = read_text_content(reader)?;
                    }
                    b"tonec" => {
                        arr.meta.tone_c = read_text_content(reader)?;
                    }
                    b"toned" => {
                        arr.meta.tone_d = read_text_content(reader)?;
                    }
                    b"songNameSort" => {
                        arr.meta.song_name_sort = read_text_content(reader)?;
                    }
                    b"internalName" => {
                        arr.meta.internal_name = read_text_content(reader)?;
                    }
                    b"ebeats" => {
                        arr.ebeats = parse_ebeats(reader)?;
                    }
                    b"phrases" => {
                        arr.phrases = parse_phrases(reader)?;
                    }
                    b"phraseIterations" => {
                        arr.phrase_iterations = parse_phrase_iterations(reader)?;
                    }
                    b"linkedDiffs" => {
                        arr.linked_diffs = parse_linked_diffs(reader)?;
                    }
                    b"phraseProperties" => {
                        arr.phrase_properties = parse_phrase_properties(reader)?;
                    }
                    b"chordTemplates" => {
                        arr.chord_templates = parse_chord_templates(reader, b"chordTemplates")?;
                    }
                    b"fretHandMuteTemplates" => {
                        arr.fret_hand_mute_templates =
                            parse_chord_templates(reader, b"fretHandMuteTemplates")?;
                    }
                    b"events" => {
                        arr.events = parse_events(reader)?;
                    }
                    b"sections" => {
                        arr.sections = parse_sections(reader)?;
                    }
                    b"levels" => {
                        arr.levels = parse_levels(reader)?;
                    }
                    b"tones" => {
                        arr.tones = parse_tones(reader)?;
                    }
                    b"song" => {
                        // entering song - just continue
                    }
                    _ => {
                        let end = e.to_end().into_owned();
                        reader.read_to_end(end.name())?;
                    }
                }
            }
            XmlEvent::Empty(e) => match e.name().as_ref() {
                b"songLength" => {
                    arr.meta.song_length = get_attr(&e, b"time")
                        .map(|s| time_from_str(&s))
                        .unwrap_or(0);
                }
                b"averageTempo" => {
                    arr.meta.average_tempo = get_attr(&e, b"bpm")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(120.0);
                }
                b"tuning" => {
                    arr.meta.tuning = parse_tuning(&e);
                }
                b"arrangementProperties" => {
                    arr.meta.arrangement_properties = parse_arrangement_properties(&e);
                }
                b"ebeats" => {}
                b"phrases" => {}
                b"phraseIterations" => {}
                b"linkedDiffs" => {}
                b"phraseProperties" => {}
                b"chordTemplates" => {}
                b"fretHandMuteTemplates" => {}
                b"events" => {}
                b"sections" => {}
                b"levels" => {}
                b"tones" => {}
                b"controls" => {}
                _ => {}
            },
            XmlEvent::End(e) if e.name().as_ref() == b"song" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
    }

    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_conversion() {
        assert_eq!(time_from_str("1.500"), 1500);
        assert_eq!(time_from_str("0.000"), 0);
        assert_eq!(time_to_str(1500), "1.500");
        assert_eq!(time_to_str(0), "0.000");
    }
}
