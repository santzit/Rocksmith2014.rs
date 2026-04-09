//! Tone import/export (XML and JSON).
//!
//! Mirrors `Tone.fs` from `Rocksmith2014.Common`.

use std::collections::HashMap;
use std::path::Path;

// ─── Domain types ─────────────────────────────────────────────────────────────

/// A single gear item (amp, cabinet, pedal, rack unit).
#[derive(Debug, Clone)]
pub struct Pedal {
    /// The gear type category string (e.g. "Amps", "Cabinets", "Pedals").
    pub pedal_type: String,
    /// Knob values keyed by knob name.
    pub knob_values: HashMap<String, f32>,
    /// The unique gear key (e.g. "Amp_OrangeAD50").
    pub key: String,
    pub category: Option<String>,
    pub skin: Option<String>,
    pub skin_index: Option<f32>,
}

impl Default for Pedal {
    fn default() -> Self {
        Self {
            pedal_type: String::new(),
            knob_values: HashMap::new(),
            key: String::new(),
            category: None,
            skin: None,
            skin_index: None,
        }
    }
}

/// The full gear list for a tone.
#[derive(Debug, Clone)]
pub struct Gear {
    pub amp: Pedal,
    pub cabinet: Pedal,
    pub racks: [Option<Pedal>; 4],
    pub pre_pedals: [Option<Pedal>; 4],
    pub post_pedals: [Option<Pedal>; 4],
}

/// A Rocksmith 2014 tone.
#[derive(Debug, Clone)]
pub struct Tone {
    pub gear_list: Gear,
    pub tone_descriptors: Vec<String>,
    pub name_separator: String,
    pub volume: f64,
    pub mac_volume: Option<f64>,
    pub key: String,
    pub name: String,
    pub sort_order: Option<f32>,
}

impl Tone {
    pub const DEFAULT_NAME_SEPARATOR: &'static str = " - ";

    /// Parses a volume string into a rounded f64 (matches .NET volumeFromString).
    pub fn volume_from_str(s: &str) -> f64 {
        // Strip non-numeric characters except '-' and '.'
        // Also replace comma decimal separator
        let filtered: String = s
            .replace(',', ".")
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
            .collect();
        let v: f64 = filtered.parse().unwrap_or(0.0);
        // Round to 1 decimal place, away from zero
        let shifted = v * 10.0;
        let rounded = if shifted >= 0.0 {
            shifted.floor() + if shifted.fract() >= 0.5 { 1.0 } else { 0.0 }
        } else {
            shifted.ceil() + if (-shifted).fract() >= 0.5 { -1.0 } else { 0.0 }
        };
        rounded / 10.0
    }

    /// Formats a volume as a string with 3 decimal places (matches .NET volumeToString).
    pub fn volume_to_string(v: f64) -> String {
        format!("{:.3}", v)
    }

    /// Returns the number of active effects (non-None pre/post pedals and racks).
    pub fn get_effect_count(gear_list: &Gear) -> usize {
        gear_list.pre_pedals.iter().filter(|p| p.is_some()).count()
            + gear_list.post_pedals.iter().filter(|p| p.is_some()).count()
            + gear_list.racks.iter().filter(|p| p.is_some()).count()
    }

    /// Imports a tone from a Tone2014 XML file.
    pub fn from_xml_file<P: AsRef<Path>>(path: P) -> Result<Self, ToneError> {
        let data = std::fs::read_to_string(path)?;
        Self::from_xml_str(&data)
    }

    /// Exports a tone to a Tone2014 XML file.
    pub fn export_xml<P: AsRef<Path>>(path: P, tone: &Tone) -> Result<(), ToneError> {
        let xml = write_tone_xml(tone)?;
        std::fs::write(path, xml)?;
        Ok(())
    }

    /// Imports a tone from a JSON file (ToneDto format).
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, ToneError> {
        let data = std::fs::read_to_string(path)?;
        let dto: ToneDto = serde_json::from_str(&data)?;
        Ok(tone_from_dto(dto))
    }

    /// Exports a tone to a JSON file (ToneDto format).
    pub fn export_json<P: AsRef<Path>>(path: P, tone: &Tone) -> Result<(), ToneError> {
        let dto = tone_to_dto(tone);
        let json = serde_json::to_string_pretty(&dto)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

// ─── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ToneError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not a valid Tone2014 XML file")]
    InvalidFile,
}

impl From<quick_xml::Error> for ToneError {
    fn from(e: quick_xml::Error) -> Self {
        ToneError::Xml(e.to_string())
    }
}

// ─── XML import ───────────────────────────────────────────────────────────────

fn local_name(name: &[u8]) -> &[u8] {
    // Strip namespace prefix (e.g. "d4p1:Key" → "Key", "i:nil" → "nil")
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

/// Read all text content of the current element (already opened).
fn read_text(reader: &mut quick_xml::Reader<&[u8]>, end_tag: &[u8]) -> String {
    let mut text = String::new();
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Text(e)) => {
                if let Ok(s) = e.unescape() {
                    text.push_str(&s);
                }
            }
            Ok(quick_xml::events::Event::End(e)) if local_name(e.name().as_ref()) == end_tag => {
                break;
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    text
}

/// Skip until the closing tag matching `end_tag` (local name).
fn skip_to_end(reader: &mut quick_xml::Reader<&[u8]>, end_tag: &[u8]) {
    let mut depth = 1usize;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e)) if local_name(e.name().as_ref()) == end_tag => {
                depth += 1;
            }
            Ok(quick_xml::events::Event::End(e)) if local_name(e.name().as_ref()) == end_tag => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
}

/// Parse `<KnobValues>` element (already entered the element).
fn parse_knob_values(reader: &mut quick_xml::Reader<&[u8]>) -> HashMap<String, f32> {
    let mut map = HashMap::new();
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e))
                if local_name(e.name().as_ref()) == b"KeyValueOfstringfloat" =>
            {
                let mut k = String::new();
                let mut v = 0.0f32;
                loop {
                    match reader.read_event() {
                        Ok(quick_xml::events::Event::Start(inner)) => {
                            let ln = local_name(inner.name().as_ref()).to_vec();
                            let text = read_text(reader, &ln);
                            match ln.as_slice() {
                                b"Key" => k = text,
                                b"Value" => v = text.trim().parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        Ok(quick_xml::events::Event::End(e))
                            if local_name(e.name().as_ref()) == b"KeyValueOfstringfloat" =>
                        {
                            break
                        }
                        Ok(quick_xml::events::Event::Eof) | Err(_) => break,
                        _ => {}
                    }
                }
                if !k.is_empty() {
                    map.insert(k, v);
                }
            }
            Ok(quick_xml::events::Event::End(e))
                if local_name(e.name().as_ref()) == b"KnobValues" =>
            {
                break
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    map
}

/// Parse a `<PrePedal1>` / `<Amp>` etc element.
/// `start` is the opening event already consumed.
fn parse_pedal(
    reader: &mut quick_xml::Reader<&[u8]>,
    tag: &[u8],
) -> Option<Pedal> {
    let mut pedal = Pedal::default();
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e)) => {
                let ln = local_name(e.name().as_ref()).to_vec();
                match ln.as_slice() {
                    b"KnobValues" => {
                        pedal.knob_values = parse_knob_values(reader);
                    }
                    b"Category" => {
                        let text = read_text(reader, b"Category");
                        if !text.is_empty() {
                            pedal.category = Some(text);
                        }
                    }
                    b"PedalKey" => {
                        pedal.key = read_text(reader, b"PedalKey");
                    }
                    b"Skin" => {
                        let text = read_text(reader, b"Skin");
                        if !text.is_empty() {
                            pedal.skin = Some(text);
                        }
                    }
                    b"SkinIndex" => {
                        let text = read_text(reader, b"SkinIndex");
                        if !text.is_empty() {
                            pedal.skin_index = text.trim().parse().ok();
                        }
                    }
                    b"Type" => {
                        pedal.pedal_type = read_text(reader, b"Type");
                    }
                    _ => {
                        skip_to_end(reader, &ln);
                    }
                }
            }
            Ok(quick_xml::events::Event::Empty(e)) => {
                let ln = local_name(e.name().as_ref()).to_vec();
                // nil attributes mean None (already None by default)
                // No action needed
                let _ = ln;
            }
            Ok(quick_xml::events::Event::End(e)) if local_name(e.name().as_ref()) == tag => {
                break;
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    Some(pedal)
}

/// Parse `<GearList>` element (already entered).
fn parse_gear_list(reader: &mut quick_xml::Reader<&[u8]>) -> Result<Gear, ToneError> {
    let mut amp: Option<Pedal> = None;
    let mut cabinet: Option<Pedal> = None;
    let mut pre: [Option<Pedal>; 4] = [None, None, None, None];
    let mut post: [Option<Pedal>; 4] = [None, None, None, None];
    let mut racks: [Option<Pedal>; 4] = [None, None, None, None];

    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e)) => {
                let ln = local_name(e.name().as_ref()).to_vec();
                match ln.as_slice() {
                    b"Amp" => amp = parse_pedal(reader, b"Amp"),
                    b"Cabinet" => cabinet = parse_pedal(reader, b"Cabinet"),
                    b"PrePedal1" => pre[0] = parse_pedal(reader, b"PrePedal1"),
                    b"PrePedal2" => pre[1] = parse_pedal(reader, b"PrePedal2"),
                    b"PrePedal3" => pre[2] = parse_pedal(reader, b"PrePedal3"),
                    b"PrePedal4" => pre[3] = parse_pedal(reader, b"PrePedal4"),
                    b"PostPedal1" => post[0] = parse_pedal(reader, b"PostPedal1"),
                    b"PostPedal2" => post[1] = parse_pedal(reader, b"PostPedal2"),
                    b"PostPedal3" => post[2] = parse_pedal(reader, b"PostPedal3"),
                    b"PostPedal4" => post[3] = parse_pedal(reader, b"PostPedal4"),
                    b"Rack1" => racks[0] = parse_pedal(reader, b"Rack1"),
                    b"Rack2" => racks[1] = parse_pedal(reader, b"Rack2"),
                    b"Rack3" => racks[2] = parse_pedal(reader, b"Rack3"),
                    b"Rack4" => racks[3] = parse_pedal(reader, b"Rack4"),
                    _ => skip_to_end(reader, &ln),
                }
            }
            Ok(quick_xml::events::Event::Empty(e)) => {
                // i:nil="true" → already None by default, skip
                let _ = e;
            }
            Ok(quick_xml::events::Event::End(e))
                if local_name(e.name().as_ref()) == b"GearList" =>
            {
                break;
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    Ok(Gear {
        amp: amp.ok_or(ToneError::Xml("Missing Amp".into()))?,
        cabinet: cabinet.ok_or(ToneError::Xml("Missing Cabinet".into()))?,
        pre_pedals: pre,
        post_pedals: post,
        racks,
    })
}

/// Parse ToneDescriptors element (already entered).
fn parse_tone_descriptors(reader: &mut quick_xml::Reader<&[u8]>) -> Vec<String> {
    let mut descriptors = Vec::new();
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e))
                if local_name(e.name().as_ref()) == b"string" =>
            {
                descriptors.push(read_text(reader, b"string"));
            }
            Ok(quick_xml::events::Event::End(e))
                if local_name(e.name().as_ref()) == b"ToneDescriptors" =>
            {
                break;
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    descriptors
}

impl Tone {
    /// Parses a Tone from a Tone2014 XML string.
    pub fn from_xml_str(xml: &str) -> Result<Self, ToneError> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        // Find the root element
        loop {
            match reader.read_event()? {
                quick_xml::events::Event::Start(e)
                    if local_name(e.name().as_ref()) == b"Tone2014" =>
                {
                    break;
                }
                quick_xml::events::Event::Eof => return Err(ToneError::InvalidFile),
                _ => {}
            }
        }

        let mut gear_list: Option<Gear> = None;
        let mut tone_descriptors = Vec::new();
        let mut name_separator = Tone::DEFAULT_NAME_SEPARATOR.to_string();
        let mut volume = 0.0f64;
        let mut mac_volume: Option<f64> = None;
        let mut key = String::new();
        let mut name = String::new();
        let mut sort_order: Option<f32> = None;

        loop {
            match reader.read_event()? {
                quick_xml::events::Event::Start(e) => {
                    let ln = local_name(e.name().as_ref()).to_vec();
                    match ln.as_slice() {
                        b"GearList" => {
                            gear_list = Some(parse_gear_list(&mut reader)?);
                        }
                        b"ToneDescriptors" => {
                            tone_descriptors = parse_tone_descriptors(&mut reader);
                        }
                        b"NameSeparator" => {
                            name_separator = read_text(&mut reader, b"NameSeparator");
                        }
                        b"Volume" => {
                            let s = read_text(&mut reader, b"Volume");
                            volume = Tone::volume_from_str(&s);
                        }
                        b"MacVolume" => {
                            let s = read_text(&mut reader, b"MacVolume");
                            if !s.is_empty() {
                                mac_volume = Some(Tone::volume_from_str(&s));
                            }
                        }
                        b"Key" => {
                            key = read_text(&mut reader, b"Key");
                        }
                        b"Name" => {
                            name = read_text(&mut reader, b"Name");
                        }
                        b"SortOrder" => {
                            let s = read_text(&mut reader, b"SortOrder");
                            sort_order = s.trim().parse().ok();
                        }
                        _ => {
                            skip_to_end(&mut reader, &ln);
                        }
                    }
                }
                quick_xml::events::Event::End(e)
                    if local_name(e.name().as_ref()) == b"Tone2014" =>
                {
                    break;
                }
                quick_xml::events::Event::Eof => break,
                _ => {}
            }
        }

        Ok(Tone {
            gear_list: gear_list.ok_or(ToneError::Xml("Missing GearList".into()))?,
            tone_descriptors,
            name_separator,
            volume,
            mac_volume,
            key,
            name,
            sort_order,
        })
    }
}

// ─── XML export ───────────────────────────────────────────────────────────────

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn write_pedal_xml(buf: &mut String, tag: &str, pedal: &Pedal) {
    let ns_i = r#" xmlns:i="http://www.w3.org/2001/XMLSchema-instance""#;
    buf.push_str(&format!("    <{tag}>\n"));
    match &pedal.category {
        Some(c) => buf.push_str(&format!("      <Category>{}</Category>\n", escape_xml(c))),
        None => buf.push_str(&format!("      <Category{ns_i} i:nil=\"true\" />\n")),
    }
    // KnobValues
    if pedal.knob_values.is_empty() {
        buf.push_str(
            r#"      <KnobValues xmlns:d4p1="http://schemas.microsoft.com/2003/10/Serialization/Arrays" />
"#,
        );
    } else {
        buf.push_str(
            r#"      <KnobValues xmlns:d4p1="http://schemas.microsoft.com/2003/10/Serialization/Arrays">
"#,
        );
        // Sort keys for deterministic output
        let mut keys: Vec<&String> = pedal.knob_values.keys().collect();
        keys.sort();
        for k in keys {
            let v = pedal.knob_values[k];
            buf.push_str("        <d4p1:KeyValueOfstringfloat>\n");
            buf.push_str(&format!("          <d4p1:Key>{}</d4p1:Key>\n", escape_xml(k)));
            buf.push_str(&format!("          <d4p1:Value>{v}</d4p1:Value>\n"));
            buf.push_str("        </d4p1:KeyValueOfstringfloat>\n");
        }
        buf.push_str("      </KnobValues>\n");
    }
    buf.push_str(&format!(
        "      <PedalKey>{}</PedalKey>\n",
        escape_xml(&pedal.key)
    ));
    match &pedal.skin {
        Some(s) => buf.push_str(&format!("      <Skin>{}</Skin>\n", escape_xml(s))),
        None => buf.push_str(&format!("      <Skin{ns_i} i:nil=\"true\" />\n")),
    }
    match &pedal.skin_index {
        Some(si) => buf.push_str(&format!("      <SkinIndex>{si}</SkinIndex>\n")),
        None => buf.push_str(&format!("      <SkinIndex{ns_i} i:nil=\"true\" />\n")),
    }
    buf.push_str(&format!(
        "      <Type>{}</Type>\n",
        escape_xml(&pedal.pedal_type)
    ));
    buf.push_str(&format!("    </{tag}>\n"));
}

fn write_optional_pedal_xml(buf: &mut String, tag: &str, pedal: &Option<Pedal>) {
    let ns_i = r#" xmlns:i="http://www.w3.org/2001/XMLSchema-instance""#;
    match pedal {
        Some(p) => write_pedal_xml(buf, tag, p),
        None => buf.push_str(&format!("    <{tag}{ns_i} i:nil=\"true\" />\n")),
    }
}

fn write_tone_xml(tone: &Tone) -> Result<String, ToneError> {
    let ns_i = r#"xmlns:i="http://www.w3.org/2001/XMLSchema-instance""#;
    let ns = r#"xmlns="http://schemas.datacontract.org/2004/07/RocksmithToolkitLib.DLCPackage.Manifest2014.Tone""#;
    let mut buf = String::new();
    buf.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    buf.push('\n');
    buf.push_str(&format!("<Tone2014 {ns_i} {ns}>\n"));

    // GearList
    buf.push_str("  <GearList>\n");
    write_pedal_xml(&mut buf, "Amp", &tone.gear_list.amp);
    write_pedal_xml(&mut buf, "Cabinet", &tone.gear_list.cabinet);
    write_optional_pedal_xml(&mut buf, "PostPedal1", &tone.gear_list.post_pedals[0]);
    write_optional_pedal_xml(&mut buf, "PostPedal2", &tone.gear_list.post_pedals[1]);
    write_optional_pedal_xml(&mut buf, "PostPedal3", &tone.gear_list.post_pedals[2]);
    write_optional_pedal_xml(&mut buf, "PostPedal4", &tone.gear_list.post_pedals[3]);
    write_optional_pedal_xml(&mut buf, "PrePedal1", &tone.gear_list.pre_pedals[0]);
    write_optional_pedal_xml(&mut buf, "PrePedal2", &tone.gear_list.pre_pedals[1]);
    write_optional_pedal_xml(&mut buf, "PrePedal3", &tone.gear_list.pre_pedals[2]);
    write_optional_pedal_xml(&mut buf, "PrePedal4", &tone.gear_list.pre_pedals[3]);
    write_optional_pedal_xml(&mut buf, "Rack1", &tone.gear_list.racks[0]);
    write_optional_pedal_xml(&mut buf, "Rack2", &tone.gear_list.racks[1]);
    write_optional_pedal_xml(&mut buf, "Rack3", &tone.gear_list.racks[2]);
    write_optional_pedal_xml(&mut buf, "Rack4", &tone.gear_list.racks[3]);
    buf.push_str("  </GearList>\n");

    buf.push_str("  <IsCustom>true</IsCustom>\n");
    buf.push_str(&format!("  <Key>{}</Key>\n", escape_xml(&tone.key)));
    buf.push_str(&format!("  <Name>{}</Name>\n", escape_xml(&tone.name)));
    buf.push_str(&format!(
        "  <NameSeparator>{}</NameSeparator>\n",
        escape_xml(&tone.name_separator)
    ));
    buf.push_str(&format!(
        "  <SortOrder>{}</SortOrder>\n",
        tone.sort_order.map(|v| format!("{v}")).unwrap_or_else(|| "0".into())
    ));

    // ToneDescriptors
    buf.push_str(
        r#"  <ToneDescriptors xmlns:d2p1="http://schemas.microsoft.com/2003/10/Serialization/Arrays">
"#,
    );
    for d in &tone.tone_descriptors {
        buf.push_str(&format!("    <d2p1:string>{}</d2p1:string>\n", escape_xml(d)));
    }
    buf.push_str("  </ToneDescriptors>\n");

    buf.push_str(&format!(
        "  <Volume>{}</Volume>\n",
        Tone::volume_to_string(tone.volume)
    ));
    buf.push_str("</Tone2014>\n");

    Ok(buf)
}

// ─── JSON DTO types ────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PedalDto {
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none", default)]
    pedal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    knob_values: Option<HashMap<String, f32>>,
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none", default)]
    pedal_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    skin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    skin_index: Option<f32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GearDto {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rack1: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rack2: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rack3: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rack4: Option<PedalDto>,
    amp: PedalDto,
    cabinet: PedalDto,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pre_pedal1: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pre_pedal2: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pre_pedal3: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pre_pedal4: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    post_pedal1: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    post_pedal2: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    post_pedal3: Option<PedalDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    post_pedal4: Option<PedalDto>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ToneDto {
    gear_list: GearDto,
    #[serde(default)]
    tone_descriptors: Vec<String>,
    #[serde(default = "default_separator")]
    name_separator: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    is_custom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    mac_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    sort_order: Option<f32>,
}

fn default_separator() -> String {
    Tone::DEFAULT_NAME_SEPARATOR.to_string()
}

// ─── DTO conversions ──────────────────────────────────────────────────────────

fn pedal_from_dto(dto: PedalDto) -> Pedal {
    Pedal {
        pedal_type: dto.pedal_type.unwrap_or_default(),
        knob_values: dto.knob_values.unwrap_or_default(),
        key: dto.pedal_key.unwrap_or_default(),
        category: dto.category,
        skin: dto.skin,
        skin_index: dto.skin_index,
    }
}

fn pedal_to_dto(pedal: &Pedal) -> PedalDto {
    PedalDto {
        pedal_type: Some(pedal.pedal_type.clone()),
        knob_values: Some(pedal.knob_values.clone()),
        pedal_key: Some(pedal.key.clone()),
        category: pedal.category.clone(),
        skin: pedal.skin.clone(),
        skin_index: pedal.skin_index,
    }
}

fn tone_from_dto(dto: ToneDto) -> Tone {
    let g = dto.gear_list;

    let opt_pedal = |p: Option<PedalDto>| p.map(pedal_from_dto);

    let gear = Gear {
        amp: pedal_from_dto(g.amp),
        cabinet: pedal_from_dto(g.cabinet),
        pre_pedals: [
            opt_pedal(g.pre_pedal1),
            opt_pedal(g.pre_pedal2),
            opt_pedal(g.pre_pedal3),
            opt_pedal(g.pre_pedal4),
        ],
        post_pedals: [
            opt_pedal(g.post_pedal1),
            opt_pedal(g.post_pedal2),
            opt_pedal(g.post_pedal3),
            opt_pedal(g.post_pedal4),
        ],
        racks: [
            opt_pedal(g.rack1),
            opt_pedal(g.rack2),
            opt_pedal(g.rack3),
            opt_pedal(g.rack4),
        ],
    };

    let volume = dto
        .volume
        .as_deref()
        .map(Tone::volume_from_str)
        .unwrap_or(0.0);
    let mac_volume = dto
        .mac_volume
        .as_deref()
        .map(Tone::volume_from_str);

    Tone {
        gear_list: gear,
        tone_descriptors: dto.tone_descriptors,
        name_separator: dto.name_separator,
        volume,
        mac_volume,
        key: dto.key.unwrap_or_default(),
        name: dto.name.unwrap_or_default(),
        sort_order: dto.sort_order,
    }
}

fn tone_to_dto(tone: &Tone) -> ToneDto {
    let try_get = |arr: &[Option<Pedal>; 4], i: usize| -> Option<PedalDto> {
        arr[i].as_ref().map(pedal_to_dto)
    };

    let gear = GearDto {
        amp: pedal_to_dto(&tone.gear_list.amp),
        cabinet: pedal_to_dto(&tone.gear_list.cabinet),
        pre_pedal1: try_get(&tone.gear_list.pre_pedals, 0),
        pre_pedal2: try_get(&tone.gear_list.pre_pedals, 1),
        pre_pedal3: try_get(&tone.gear_list.pre_pedals, 2),
        pre_pedal4: try_get(&tone.gear_list.pre_pedals, 3),
        post_pedal1: try_get(&tone.gear_list.post_pedals, 0),
        post_pedal2: try_get(&tone.gear_list.post_pedals, 1),
        post_pedal3: try_get(&tone.gear_list.post_pedals, 2),
        post_pedal4: try_get(&tone.gear_list.post_pedals, 3),
        rack1: try_get(&tone.gear_list.racks, 0),
        rack2: try_get(&tone.gear_list.racks, 1),
        rack3: try_get(&tone.gear_list.racks, 2),
        rack4: try_get(&tone.gear_list.racks, 3),
    };

    let mac_vol = tone
        .mac_volume
        .map(Tone::volume_to_string);

    ToneDto {
        gear_list: gear,
        tone_descriptors: tone.tone_descriptors.clone(),
        name_separator: tone.name_separator.clone(),
        is_custom: Some(true),
        volume: Some(Tone::volume_to_string(tone.volume)),
        mac_volume: mac_vol,
        key: Some(tone.key.clone()),
        name: Some(tone.name.clone()),
        sort_order: tone.sort_order,
    }
}
