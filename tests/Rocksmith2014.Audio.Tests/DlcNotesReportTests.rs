use rocksmith2014_psarc::Psarc;
use rocksmith2014_xml::InstrumentalArrangement;
use std::fs;
use std::path::{Path, PathBuf};

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_target_song(psarc_path: &Path, entry: &str, title: &str) -> bool {
    let title_norm = normalized(title);
    let entry_norm = normalized(entry);
    let file_norm = normalized(&psarc_path.to_string_lossy());
    let has_in_between = title_norm.contains("inbetween")
        || entry_norm.contains("inbetween")
        || file_norm.contains("inbetween");
    let has_dream = title_norm.contains("dream")
        || entry_norm.contains("dream")
        || file_norm.contains("dream");
    let has_days =
        title_norm.contains("days") || entry_norm.contains("days") || file_norm.contains("days");
    has_in_between && (has_dream || has_days)
}

fn note_checksum(level: &rocksmith2014_xml::Level) -> u64 {
    let mut note_rows: Vec<String> = level
        .notes
        .iter()
        .map(|note| {
            format!(
                "{}:{}:{}:{}",
                note.time, note.string, note.fret, note.sustain
            )
        })
        .collect();
    note_rows.sort();

    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for row in note_rows {
        for byte in row.bytes().chain(std::iter::once(b';')) {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
    }
    hash
}

#[test]
fn checks_all_dlc_notes_and_prints_in_between_dreams_notes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../..");
    let dlc_dir = repo_root.join("tests/DLC");

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

    let mut checked_notes = 0usize;
    let mut target_printed = false;
    let mut summary_lines = Vec::new();

    for psarc_path in &psarc_files {
        let psarc_name = psarc_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let mut psarc = Psarc::open(psarc_path).expect("open psarc");
        let mut xml_entries: Vec<String> = psarc
            .manifest()
            .iter()
            .filter(|entry| entry.ends_with(".xml"))
            .cloned()
            .collect();
        xml_entries.sort();

        for xml_entry in xml_entries {
            let xml_bytes = match psarc.inflate_file(&xml_entry) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let xml_text = match std::str::from_utf8(&xml_bytes) {
                Ok(text) => text,
                Err(_) => continue,
            };
            let arrangement = match InstrumentalArrangement::from_xml(xml_text) {
                Ok(arr) => arr,
                Err(_) => continue,
            };

            for level in &arrangement.levels {
                checked_notes += level.notes.len();
                summary_lines.push(format!(
                    "{psarc_name}\t{xml_entry}\t{}\t{}\t{:016x}",
                    level.difficulty,
                    level.notes.len(),
                    note_checksum(level)
                ));
            }

            let has_any_notes = arrangement.levels.iter().any(|level| !level.notes.is_empty());
            if has_any_notes && is_target_song(psarc_path, &xml_entry, &arrangement.meta.song_name) {
                target_printed = true;
                println!(
                    "\nTarget song: {} ({})",
                    arrangement.meta.song_name,
                    psarc_path.display()
                );
                println!("Arrangement file: {xml_entry}");
                for level in &arrangement.levels {
                    println!(
                        "Difficulty {} ({} notes)",
                        level.difficulty,
                        level.notes.len()
                    );
                    for note in &level.notes {
                        println!(
                            "  time={} string={} fret={} sustain={}",
                            note.time, note.string, note.fret, note.sustain
                        );
                    }
                }
            }
        }
    }

    if let Ok(summary_path) = std::env::var("RUST_DLC_NOTE_SUMMARY_PATH") {
        summary_lines.sort();
        fs::write(summary_path, summary_lines.join("\n")).expect("write rust DLC note summary");
    }

    assert!(checked_notes > 0, "no notes found across DLC xml files");
    assert!(
        target_printed,
        "no DLC arrangement matched 'In between dreams' search"
    );
}
