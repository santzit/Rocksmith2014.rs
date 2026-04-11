//! DLC WEM duration checks.
//!
//! Verifies that all MAIN WEM files contained in `tests/DLC/*.psarc`
//! decode to at least one minute of audio.
//!
//! For each PSARC the test:
//!  1. Parses every `.bnk` (Wwise SoundBank) file to map WEM IDs to their role
//!     (`MAIN` or `PREVIEW`) — preview banks have `_preview` in their name.
//!  2. Parses the `.hsan` manifest to extract artist/song metadata.
//!  3. Decodes every `.wem` entry and prints rich metadata.
//!  4. FAILs if any MAIN-track WEM is shorter than 60 seconds.

use rocksmith2014_audio::conversion;
use rocksmith2014_psarc::Psarc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const MIN_REQUIRED_DURATION_SECS: f64 = 60.0;

// ---------------------------------------------------------------------------
// BNK helpers
// ---------------------------------------------------------------------------

/// Extract the list of external WEM IDs referenced by a Wwise SoundBank's
/// DIDX section.  Each DIDX entry is 12 bytes: id (u32) + offset (u32) + len (u32).
fn wem_ids_from_bnk(data: &[u8]) -> Vec<u32> {
    let mut ids = Vec::new();
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let tag = &data[i..i + 4];
        let size = u32::from_le_bytes(data[i + 4..i + 8].try_into().unwrap_or([0; 4])) as usize;
        if tag == b"DIDX" {
            let mut j = i + 8;
            while j + 12 <= i + 8 + size {
                let wem_id = u32::from_le_bytes(data[j..j + 4].try_into().unwrap_or([0; 4]));
                ids.push(wem_id);
                j += 12;
            }
            break;
        }
        i += 8 + size;
        if size == 0 {
            break;
        }
    }
    ids
}

/// Build a map from WEM numeric ID → role string ("MAIN" or "PREVIEW") by
/// reading every `.bnk` entry in the archive.
///
/// MAIN takes priority: if both a main and a preview bank reference the same
/// WEM (e.g. older DLCs that reuse one file for both), it is treated as MAIN.
fn build_wem_role_map(psarc: &mut Psarc<std::fs::File>) -> HashMap<u32, &'static str> {
    let bnk_entries: Vec<String> = psarc
        .manifest()
        .iter()
        .filter(|n| n.ends_with(".bnk"))
        .cloned()
        .collect();

    let mut map: HashMap<u32, &'static str> = HashMap::new();
    // Process MAIN banks first so they win if the same ID appears in preview.
    let (main_bnks, preview_bnks): (Vec<_>, Vec<_>) = bnk_entries
        .into_iter()
        .partition(|e| !e.to_ascii_lowercase().contains("_preview"));

    for bnk_entry in main_bnks.iter().chain(preview_bnks.iter()) {
        let is_preview = bnk_entry.to_ascii_lowercase().contains("_preview");
        let role: &'static str = if is_preview { "PREVIEW" } else { "MAIN" };
        if let Ok(data) = psarc.inflate_file(bnk_entry) {
            for id in wem_ids_from_bnk(&data) {
                // Only insert if not already set to MAIN (MAIN wins).
                map.entry(id).or_insert(role);
                if !is_preview {
                    map.insert(id, "MAIN");
                }
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// HSAN / JSON manifest helpers
// ---------------------------------------------------------------------------

/// Minimal song metadata extracted from the PSARC's `.hsan` manifest.
#[derive(Default)]
struct SongMeta {
    artist: String,
    title: String,
    song_length_secs: Option<f64>,
}

/// Parse a bare-minimum subset of the `.hsan` JSON to get artist / title /
/// length.  Uses only the standard library — no external JSON crate needed.
fn parse_hsan_meta(data: &[u8]) -> SongMeta {
    let text = String::from_utf8_lossy(data);
    let mut meta = SongMeta::default();

    if let Some(v) = json_str_field(&text, "ArtistName") {
        meta.artist = v;
    }
    if let Some(v) = json_str_field(&text, "SongName") {
        meta.title = v;
    }
    if let Some(v) = json_num_field(&text, "SongLength") {
        meta.song_length_secs = Some(v);
    }
    meta
}

/// Extract the first occurrence of `"key": "value"` from raw JSON text.
fn json_str_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = text.find(&needle)? + needle.len();
    let rest = text[start..].trim_start();
    if rest.starts_with('"') {
        let inner = &rest[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        None
    }
}

/// Extract the first occurrence of `"key": <number>` from raw JSON text.
fn json_num_field(text: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let start = text.find(&needle)? + needle.len();
    let rest = text[start..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')?;
    rest[..end].parse().ok()
}

// ---------------------------------------------------------------------------
// Misc helpers
// ---------------------------------------------------------------------------

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn header_hex(data: &[u8], take: usize) -> String {
    data.iter()
        .take(take)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[test]
fn all_dlc_main_wem_files_are_at_least_one_minute() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../..");
    let dlc_dir = repo_root.join("tests/DLC");
    let temp_dir = tempfile::tempdir().expect("create temp dir");

    let mut psarc_files: Vec<PathBuf> = fs::read_dir(&dlc_dir)
        .expect("read tests/DLC")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("psarc"))
        .collect();
    psarc_files.sort();

    assert!(
        !psarc_files.is_empty(),
        "no DLC psarc files found in tests/DLC"
    );

    let mut main_checked = 0usize;
    let mut failures = Vec::new();

    for psarc_path in &psarc_files {
        let mut psarc = Psarc::open(psarc_path).expect("open psarc");
        let psarc_name = psarc_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("dlc");

        // --- build WEM ID → role map from BNK files ---
        let wem_roles = build_wem_role_map(&mut psarc);

        // --- extract song metadata from the first .hsan ---
        let hsan_entries: Vec<String> = psarc
            .manifest()
            .iter()
            .filter(|n| n.ends_with(".hsan"))
            .cloned()
            .collect();
        let song_meta = hsan_entries
            .first()
            .and_then(|e| psarc.inflate_file(e).ok())
            .map(|d| parse_hsan_meta(&d))
            .unwrap_or_default();

        // --- print PSARC header ---
        println!();
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║  PSARC : {}", psarc_name);
        if !song_meta.artist.is_empty() {
            println!("║  Artist: {}", song_meta.artist);
            println!("║  Title : {}", song_meta.title);
            if let Some(len) = song_meta.song_length_secs {
                println!("║  Length: {:.2} s  (from manifest)", len);
            }
        }
        println!("╚══════════════════════════════════════════════════════╝");

        // --- all WEM entries ---
        let wem_entries: Vec<String> = psarc
            .manifest()
            .iter()
            .filter(|name| name.ends_with(".wem"))
            .cloned()
            .collect();

        for (index, wem_entry) in wem_entries.into_iter().enumerate() {
            let wem_data = psarc.inflate_file(&wem_entry).expect("inflate wem");
            let wem_size_bytes = wem_data.len();
            let wem_file_name = Path::new(&wem_entry)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("audio.wem");

            // Determine role from BNK map
            let wem_id: Option<u32> = wem_file_name.trim_end_matches(".wem").parse().ok();
            let role = wem_id
                .and_then(|id| wem_roles.get(&id).copied())
                .unwrap_or("UNKNOWN");

            let wem_magic = if wem_data.len() >= 4 {
                std::str::from_utf8(&wem_data[0..4]).unwrap_or("????")
            } else {
                "????"
            };

            // Write temp WEM and decode to WAV
            let base_name = format!(
                "{}_{}_{}_{}",
                psarc_name,
                index,
                sanitize_filename(&wem_entry),
                sanitize_filename(wem_file_name)
            );
            let wem_path = temp_dir.path().join(base_name);
            fs::write(&wem_path, &wem_data).expect("write temp wem");

            conversion::wem_to_wav(&wem_path).expect("convert wem to wav");
            let wav_path = wem_path.with_extension("wav");
            let wav_reader = hound::WavReader::open(&wav_path).expect("open converted wav");
            let spec = wav_reader.spec();
            let total_samples = wav_reader.duration() as u64;
            let duration_secs = total_samples as f64 / spec.sample_rate as f64;

            // Verdict
            let verdict = if role == "MAIN" && duration_secs < MIN_REQUIRED_DURATION_SECS {
                "FAIL  ← MAIN track must be ≥ 60 s"
            } else if role == "PREVIEW" {
                "OK    (preview — short duration expected)"
            } else if role == "UNKNOWN" && duration_secs < MIN_REQUIRED_DURATION_SECS {
                "FAIL  ← unknown role, duration < 60 s"
            } else {
                "SUCCESS"
            };

            println!();
            println!("  --- WEM #{} ---", index + 1);
            println!("  Entry path:       {}", wem_entry);
            println!("  File name:        {}", wem_file_name);
            println!("  Role:             {}", role);
            println!("  WEM size:         {} bytes", wem_size_bytes);
            println!("  Header magic:     {}", wem_magic);
            println!("  Header bytes:     {}", header_hex(&wem_data, 16));
            println!("  Sample rate:      {} Hz", spec.sample_rate);
            println!("  Channels:         {}", spec.channels);
            println!("  Total samples:    {}", total_samples);
            println!("  Duration:         {:.2} s", duration_secs);
            println!("  Result:           {}", verdict);

            if role != "PREVIEW" {
                main_checked += 1;
                if duration_secs < MIN_REQUIRED_DURATION_SECS {
                    failures.push(format!(
                        "[{}] {} :: {} (role={}, duration={:.2}s)",
                        psarc_name,
                        psarc_path.display(),
                        wem_entry,
                        role,
                        duration_secs
                    ));
                }
            }
        }
    }

    assert!(
        main_checked > 0,
        "no non-preview WEM files were found in any PSARC archives under tests/DLC"
    );
    assert!(
        failures.is_empty(),
        "MAIN WEM duration check failed for:\n{}",
        failures.join("\n")
    );
}
