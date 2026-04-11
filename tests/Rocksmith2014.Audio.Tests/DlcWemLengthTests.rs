//! DLC WEM duration checks.
//!
//! Verifies that all non-preview WEM files contained in `tests/DLC/*.psarc`
//! decode to at least one minute of audio.

use rocksmith2014_audio::conversion;
use rocksmith2014_psarc::Psarc;
use std::fs;
use std::path::{Path, PathBuf};

fn is_preview_wem(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with("_preview.wem") || lower.contains("/preview/")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[test]
#[ignore = "manual DLC validation script; run with --ignored --nocapture"]
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

    assert!(!psarc_files.is_empty(), "no DLC psarc files found in tests/DLC");

    let mut checked = 0usize;
    let mut failures = Vec::new();

    for psarc_path in psarc_files {
        let mut psarc = Psarc::open(&psarc_path).expect("open psarc");
        let wem_entries: Vec<String> = psarc
            .manifest()
            .iter()
            .filter(|name| name.ends_with(".wem") && !is_preview_wem(name))
            .cloned()
            .collect();

        for wem_entry in wem_entries {
            checked += 1;

            let wem_data = psarc.inflate_file(&wem_entry).expect("inflate wem");
            let psarc_name = psarc_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("dlc");
            let wem_file_name = Path::new(&wem_entry)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("audio.wem");
            let base_name = format!("{}_{}", psarc_name, sanitize_filename(wem_file_name));
            let wem_path = temp_dir.path().join(base_name);
            fs::write(&wem_path, wem_data).expect("write temp wem");

            conversion::wem_to_wav(&wem_path).expect("convert wem to wav");
            let wav_path = wem_path.with_extension("wav");
            let wav_reader = hound::WavReader::open(&wav_path).expect("open converted wav");
            let spec = wav_reader.spec();
            let total_samples = wav_reader.duration() as u64;
            let duration_secs = total_samples as f64 / spec.sample_rate as f64;

            println!("=== WEM: {} ===", wem_entry);
            println!("  Sample rate:      {} Hz", spec.sample_rate);
            println!("  Channels:         {}", spec.channels);
            println!("  Total samples:    {}", total_samples);
            println!("  Duration:         {:.2} s", duration_secs);

            if duration_secs < 60.0 {
                println!("  FAIL");
                failures.push(format!(
                    "{} :: {} ({:.2}s)",
                    psarc_path.display(),
                    wem_entry,
                    duration_secs
                ));
            } else {
                println!("  SUCCESS");
            }
        }
    }

    assert!(checked > 0, "no non-preview WEM files were found in tests/DLC");
    assert!(
        failures.is_empty(),
        "WEM duration check failed for:\n{}",
        failures.join("\n")
    );
}
