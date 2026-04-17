//! CLI shim for Rust/.NET note-parity comparison.
//!
//! # Usage
//!
//! ```text
//! rocksmith2014-ffi psarc-notes <psarc_path>
//! ```
//!
//! Iterates every `.xml` entry inside the PSARC, parses each as an
//! `InstrumentalArrangement`, and prints one tab-separated line per difficulty
//! level:
//!
//! ```text
//! <psarc_filename>\t<xml_entry>\t<difficulty>\t<note_count>\t<checksum_hex>
//! ```
//!
//! Lines are sorted lexicographically before output so they can be diffed
//! directly against .NET output.  The checksum is the same FNV-1a digest used
//! in `DlcNotesReportTests`.

use rocksmith2014_psarc::Psarc;
use rocksmith2014_xml::InstrumentalArrangement;
use std::env;
use std::path::Path;
use std::process;

/// FNV-1a checksum over sorted `"time:string:fret:sustain"` note tuples,
/// matching the algorithm in `DlcNotesReportTests.rs` and `NoteCheck.cs`.
fn note_checksum(level: &rocksmith2014_xml::Level) -> u64 {
    let mut rows: Vec<String> = level
        .notes
        .iter()
        .map(|n| format!("{}:{}:{}:{}", n.time, n.string, n.fret, n.sustain))
        .collect();
    rows.sort();

    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for row in rows {
        for byte in row.bytes().chain(std::iter::once(b';')) {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

fn cmd_psarc_notes(psarc_path: &str) {
    let path = Path::new(psarc_path);
    let psarc_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(psarc_path);

    let mut psarc = match Psarc::open(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error opening {psarc_path}: {e}");
            process::exit(1);
        }
    };

    let mut xml_entries: Vec<String> = psarc
        .manifest()
        .iter()
        .filter(|e| e.ends_with(".xml"))
        .cloned()
        .collect();
    xml_entries.sort();

    let mut lines: Vec<String> = Vec::new();

    for xml_entry in xml_entries {
        let bytes = match psarc.inflate_file(&xml_entry) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let text = match std::str::from_utf8(&bytes) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let arr = match InstrumentalArrangement::from_xml(text) {
            Ok(a) => a,
            Err(_) => continue,
        };
        for level in &arr.levels {
            lines.push(format!(
                "{psarc_name}\t{xml_entry}\t{}\t{}\t{:016x}",
                level.difficulty,
                level.notes.len(),
                note_checksum(level)
            ));
        }
    }

    lines.sort();
    for line in lines {
        println!("{line}");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("psarc-notes") => {
            let path = match args.get(2) {
                Some(p) => p.as_str(),
                None => {
                    eprintln!("usage: rocksmith2014-ffi psarc-notes <psarc_path>");
                    process::exit(1);
                }
            };
            cmd_psarc_notes(path);
        }
        _ => {
            eprintln!("usage: rocksmith2014-ffi psarc-notes <psarc_path>");
            process::exit(1);
        }
    }
}
