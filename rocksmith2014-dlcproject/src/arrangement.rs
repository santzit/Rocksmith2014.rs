use rocksmith2014_common::{random, AudioFile};
use rocksmith2014_xml::MetaData;
use uuid::Uuid;

/// The name of an instrumental arrangement path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangementName {
    Lead = 0,
    Combo = 1,
    Rhythm = 2,
    Bass = 3,
}

/// The route mask for an instrumental arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMask {
    None = 0,
    Lead = 1,
    Rhythm = 2,
    Any = 3,
    Bass = 4,
}

/// The priority of an arrangement (Main, Alternative, or Bonus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangementPriority {
    Main = 0,
    Alternative = 1,
    Bonus = 2,
}

/// An instrumental (guitar/bass) arrangement.
#[derive(Debug, Clone)]
pub struct Instrumental {
    pub id: Uuid,
    pub xml_path: String,
    pub name: ArrangementName,
    pub route_mask: RouteMask,
    pub priority: ArrangementPriority,
    pub scroll_speed: f64,
    pub bass_picked: bool,
    pub tuning: [i16; 6],
    pub tuning_pitch: f64,
    pub base_tone: String,
    pub tones: Vec<String>,
    pub custom_audio: Option<AudioFile>,
    pub master_id: i32,
    pub persistent_id: Uuid,
}

impl Default for Instrumental {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Instrumental {
            id,
            xml_path: String::new(),
            name: ArrangementName::Lead,
            route_mask: RouteMask::Lead,
            priority: ArrangementPriority::Main,
            scroll_speed: 1.3,
            bass_picked: false,
            tuning: [0; 6],
            tuning_pitch: 440.0,
            base_tone: String::new(),
            tones: Vec::new(),
            custom_audio: None,
            master_id: 0,
            persistent_id: id,
        }
    }
}

/// A vocals arrangement.
#[derive(Debug, Clone)]
pub struct Vocals {
    pub id: Uuid,
    pub xml_path: String,
    pub japanese: bool,
    pub custom_font: Option<String>,
    pub master_id: i32,
    pub persistent_id: Uuid,
}

/// A showlights arrangement.
#[derive(Debug, Clone)]
pub struct Showlights {
    pub id: Uuid,
    pub xml_path: String,
}

/// An arrangement of any type.
#[derive(Debug, Clone)]
pub enum Arrangement {
    Instrumental(Instrumental),
    Vocals(Vocals),
    Showlights(Showlights),
}

/// An error that can occur when loading an arrangement from a file.
#[derive(Debug)]
pub enum ArrangementLoadError {
    /// The file is not a recognised arrangement type.
    UnknownArrangement(String),
    /// Parsing the file failed with an exception.
    FailedWithException(String, Box<dyn std::error::Error>),
    /// The file is an EOF EXT vocals file that should not be imported.
    EofExtVocalsFile(String),
}

impl std::fmt::Display for ArrangementLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownArrangement(p) => write!(f, "Unknown arrangement: {}", p),
            Self::FailedWithException(p, e) => write!(f, "Failed loading '{}': {}", p, e),
            Self::EofExtVocalsFile(p) => write!(f, "EOF EXT vocals file: {}", p),
        }
    }
}

impl std::error::Error for ArrangementLoadError {}

/// Returns the root element name of an XML file, or an error if the file
/// cannot be read or parsed.
fn read_root_element(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    loop {
        match reader.read_event()? {
            Event::Start(e) | Event::Empty(e) => {
                return Ok(String::from_utf8_lossy(e.local_name().as_ref()).into_owned());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Err("No root element found in XML file".into())
}

/// Returns `true` if the file-stem ends with `_EXT` (case-sensitive).
fn is_eof_ext_path(path: &str) -> bool {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with("_EXT"))
        .unwrap_or(false)
}

/// Loads an arrangement from a file.
///
/// `create_base_tone_name` is called for an instrumental arrangement when the
/// file does not contain a `<tonebase>` element.
pub fn from_file<F>(
    create_base_tone_name: F,
    path: &str,
) -> Result<(Arrangement, Option<MetaData>), ArrangementLoadError>
where
    F: Fn(&MetaData, RouteMask) -> String,
{
    // EOF EXT vocals files must not be imported; check the filename first so
    // that we never need to attempt parsing them.
    if is_eof_ext_path(path) {
        return Err(ArrangementLoadError::EofExtVocalsFile(path.to_string()));
    }

    let root = read_root_element(path).map_err(|e| {
        ArrangementLoadError::FailedWithException(path.to_string(), e)
    })?;

    match root.as_str() {
        "song" => load_instrumental(create_base_tone_name, path),
        "vocals" => load_vocals(path),
        "showlights" => load_showlights(path),
        _ => Err(ArrangementLoadError::UnknownArrangement(path.to_string())),
    }
}

fn load_instrumental<F>(
    create_base_tone_name: F,
    path: &str,
) -> Result<(Arrangement, Option<MetaData>), ArrangementLoadError>
where
    F: Fn(&MetaData, RouteMask) -> String,
{
    let arr = rocksmith2014_xml::read_file(path)
        .map_err(|e| ArrangementLoadError::FailedWithException(path.to_string(), Box::new(e)))?;

    let meta = &arr.meta;
    let ap = &meta.arrangement_properties;

    let route_mask = if ap.path_bass != 0 {
        RouteMask::Bass
    } else if ap.path_rhythm != 0 {
        RouteMask::Rhythm
    } else {
        RouteMask::Lead
    };

    let base_tone = if meta.tone_base.is_empty() {
        create_base_tone_name(meta, route_mask)
    } else {
        meta.tone_base.clone()
    };

    // Collect non-empty tone names (tone_a through tone_d)
    let tones: Vec<String> = [&meta.tone_a, &meta.tone_b, &meta.tone_c, &meta.tone_d]
        .iter()
        .filter(|&&s| !s.is_empty())
        .map(|&s| s.clone())
        .collect();

    let name = match meta.arrangement.as_str() {
        "Lead" => ArrangementName::Lead,
        "Combo" => ArrangementName::Combo,
        "Rhythm" => ArrangementName::Rhythm,
        "Bass" => ArrangementName::Bass,
        _ => match route_mask {
            RouteMask::Bass => ArrangementName::Bass,
            RouteMask::Rhythm => ArrangementName::Rhythm,
            _ => ArrangementName::Lead,
        },
    };

    let priority = if ap.represent != 0 {
        ArrangementPriority::Main
    } else if ap.bonus_arr != 0 {
        ArrangementPriority::Bonus
    } else {
        ArrangementPriority::Alternative
    };

    // For bass arrangements, mirror string 3 tuning onto strings 4 and 5
    // (EOF does not set the upper strings correctly for drop tunings)
    let tuning = if route_mask == RouteMask::Bass {
        let fourth = meta.tuning.strings[3];
        let mut t = meta.tuning.strings;
        t[4] = fourth;
        t[5] = fourth;
        t
    } else {
        meta.tuning.strings
    };

    let tuning_pitch = crate::utils::cents_to_tuning_pitch(meta.cent_offset);

    let id = Uuid::new_v4();
    let inst = Instrumental {
        id,
        xml_path: path.to_string(),
        name,
        route_mask,
        priority,
        scroll_speed: 1.3,
        bass_picked: ap.bass_pick != 0,
        tuning,
        tuning_pitch,
        base_tone,
        tones,
        custom_audio: None,
        master_id: random::next(),
        persistent_id: id,
    };

    Ok((Arrangement::Instrumental(inst), Some(arr.meta)))
}

fn load_vocals(path: &str) -> Result<(Arrangement, Option<MetaData>), ArrangementLoadError> {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Infer Japanese lyrics from the filename (case-insensitive regex j.?(vocal|lyric))
    let lower = filename.to_ascii_lowercase();
    let is_japanese = lower
        .find("vocal")
        .or_else(|| lower.find("lyric"))
        .map(|pos| {
            // Check for a 'j' one or two characters before "vocal"/"lyric"
            let prefix = &lower[..pos];
            prefix.ends_with('j') || prefix.ends_with("j_")
        })
        .unwrap_or(false);

    let custom_font = if is_japanese {
        let dir = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let font_path = dir.join("lyrics.dds");
        if font_path.exists() {
            Some(font_path.to_string_lossy().into_owned())
        } else {
            None
        }
    } else {
        None
    };

    let id = Uuid::new_v4();
    let vocals = Vocals {
        id,
        xml_path: path.to_string(),
        japanese: is_japanese,
        custom_font,
        master_id: random::next(),
        persistent_id: id,
    };
    Ok((Arrangement::Vocals(vocals), None))
}

fn load_showlights(path: &str) -> Result<(Arrangement, Option<MetaData>), ArrangementLoadError> {
    let id = Uuid::new_v4();
    let sl = Showlights {
        id,
        xml_path: path.to_string(),
    };
    Ok((Arrangement::Showlights(sl), None))
}
