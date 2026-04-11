//! DLC WEM duration checks.
//!
//! Verifies that all WEM files contained in `tests/DLC/*.psarc`
//! decode to at least one minute of audio.

use rocksmith2014_audio::conversion;
use rocksmith2014_psarc::Psarc;
use std::fs;
use std::path::{Path, PathBuf};

const MIN_REQUIRED_DURATION_SECS: f64 = 60.0;

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

    let mut checked = 0usize;
    let mut failures = Vec::new();

    for psarc_path in psarc_files {
        let mut psarc = Psarc::open(&psarc_path).expect("open psarc");
        let wem_entries: Vec<String> = psarc
            .manifest()
            .iter()
            .filter(|name| name.ends_with(".wem"))
            .cloned()
            .collect();

        for (index, wem_entry) in wem_entries.into_iter().enumerate() {
            let wem_data = psarc.inflate_file(&wem_entry).expect("inflate wem");
            let wem_size_bytes = wem_data.len();
            let wem_magic = if wem_data.len() >= 4 {
                std::str::from_utf8(&wem_data[0..4]).unwrap_or("????")
            } else {
                "????"
            };
            let psarc_name = psarc_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("dlc");
            let wem_file_name = Path::new(&wem_entry)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("audio.wem");
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
            // `wem_to_wav` writes output by swapping extension to `.wav`.
            let wav_path = wem_path.with_extension("wav");
            let wav_reader = hound::WavReader::open(&wav_path).expect("open converted wav");
            let spec = wav_reader.spec();
            let total_samples = wav_reader.duration() as u64;
            let duration_secs = total_samples as f64 / spec.sample_rate as f64;

            println!("=== WEM: {} ===", wem_entry);
            println!("  Track/package:    {}", psarc_name);
            println!("  File name:        {}", wem_file_name);
            println!("  WEM bytes:        {}", wem_size_bytes);
            println!("  Header magic:     {}", wem_magic);
            println!("  Header bytes:     {}", header_hex(&wem_data, 16));
            println!("  Sample rate:      {} Hz", spec.sample_rate);
            println!("  Channels:         {}", spec.channels);
            println!("  Total samples:    {}", total_samples);
            println!("  Duration:         {:.2} s", duration_secs);

            checked += 1;

            if duration_secs < MIN_REQUIRED_DURATION_SECS {
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

    assert!(
        checked > 0,
        "no WEM files were found in any PSARC archives under tests/DLC"
    );
    assert!(
        failures.is_empty(),
        "WEM duration check failed for:\n{}",
        failures.join("\n")
    );
}
