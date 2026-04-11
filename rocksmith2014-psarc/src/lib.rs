//! Rust implementation of the Rocksmith 2014 PSARC archive format.
//!
//! PSARC is a proprietary archive format used by Rocksmith 2014 to package
//! downloadable content (DLC). Archives contain a manifest listing all file
//! paths and a table of contents (TOC) with per-entry metadata. The TOC may
//! optionally be encrypted with AES-256 in CFB-128 mode. Individual file data
//! is split into 64 KB blocks and compressed with zlib.
//!
//! # SNG files inside a PSARC
//!
//! SNG arrangement files stored in a PSARC are *double-layered*:
//!
//! ```text
//! PSARC block-zlib  →  AES-256-CTR encrypted SNG  →  inner zlib  →  raw binary
//! ```
//!
//! [`Psarc::inflate_file`] only strips the outer PSARC block-zlib layer.
//! The returned bytes for a `.sng` entry are **still AES-256-CTR encrypted**.
//! To decode them you must use `rocksmith2014_sng::Sng::from_encrypted`, **not**
//! `Sng::read` (which expects raw, unencrypted binary):
//!
//! ```ignore
//! // rocksmith2014_sng = "0.1"  must be in your Cargo.toml
//! use rocksmith2014_psarc::Psarc;
//! use rocksmith2014_sng::{Sng, Platform};
//! use std::fs::File;
//!
//! let file = File::open("song.psarc").unwrap();
//! let mut psarc = Psarc::read(file).unwrap();
//!
//! // inflate_file returns the AES-encrypted SNG blob — NOT raw binary.
//! let encrypted_sng = psarc.inflate_file("songs/bin/generic/song_lead.sng").unwrap();
//!
//! // Decrypt + decompress with the correct platform key:
//! let sng = Sng::from_encrypted(&encrypted_sng, Platform::Pc).unwrap();
//! println!("Levels: {}", sng.levels.len());
//! ```
//!
//! # Reading
//!
//! ```no_run
//! use rocksmith2014_psarc::Psarc;
//! use std::fs::File;
//!
//! let file = File::open("song.psarc").unwrap();
//! let mut psarc = Psarc::read(file).unwrap();
//!
//! println!("Files: {:?}", psarc.manifest());
//!
//! let data = psarc.inflate_file("songs/arr/song_lead.xml").unwrap();
//! ```
//!
//! # Writing
//!
//! ```no_run
//! use rocksmith2014_psarc::{Psarc, NamedEntry};
//! use std::fs::File;
//!
//! let entries = vec![
//!     NamedEntry { name: "readme.txt".to_string(), data: b"Hello!".to_vec() },
//! ];
//!
//! let out = File::create("archive.psarc").unwrap();
//! Psarc::create(out, true, entries).unwrap();
//! ```

mod archive;
mod crypto;
mod entry;
mod error;
mod header;

pub use archive::{NamedEntry, Psarc};
pub use entry::Entry;
pub use error::{PsarcError, Result};
pub use header::Header;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{has_zlib_header, z_type};
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Cursor;

    /// Creates a small in-memory PSARC and reads it back.
    fn round_trip(entries: Vec<NamedEntry>, encrypt: bool) -> Psarc<Cursor<Vec<u8>>> {
        let mut buf = Vec::new();
        Psarc::create(&mut buf, encrypt, entries).expect("create failed");
        Psarc::read(Cursor::new(buf)).expect("read failed")
    }

    #[test]
    fn test_empty_archive_unencrypted() {
        let psarc = round_trip(vec![], false);
        assert!(psarc.manifest().is_empty());
        assert!(psarc.toc().is_empty());
    }

    #[test]
    fn test_empty_archive_encrypted() {
        let psarc = round_trip(vec![], true);
        assert!(psarc.manifest().is_empty());
        assert!(psarc.toc().is_empty());
    }

    #[test]
    fn test_single_text_file_unencrypted() {
        let entries = vec![NamedEntry {
            name: "readme.txt".to_string(),
            data: b"Hello, Rocksmith!".to_vec(),
        }];
        let mut psarc = round_trip(entries, false);

        assert_eq!(psarc.manifest(), &["readme.txt"]);
        let data = psarc.inflate_file("readme.txt").unwrap();
        assert_eq!(data, b"Hello, Rocksmith!");
    }

    #[test]
    fn test_single_text_file_encrypted() {
        let entries = vec![NamedEntry {
            name: "data.bin".to_string(),
            data: vec![0u8; 1000],
        }];
        let mut psarc = round_trip(entries, true);

        assert_eq!(psarc.manifest(), &["data.bin"]);
        let data = psarc.inflate_file("data.bin").unwrap();
        assert_eq!(data, vec![0u8; 1000]);
    }

    #[test]
    fn test_multiple_files() {
        let entries = vec![
            NamedEntry {
                name: "a/b.txt".to_string(),
                data: b"file a".to_vec(),
            },
            NamedEntry {
                name: "c.bin".to_string(),
                data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            },
        ];
        let mut psarc = round_trip(entries, false);

        assert_eq!(psarc.manifest().len(), 2);
        assert_eq!(psarc.inflate_file("a/b.txt").unwrap(), b"file a");
        assert_eq!(
            psarc.inflate_file("c.bin").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
    }

    #[test]
    fn test_large_file_multi_block() {
        // File larger than one 64 KB block.
        let big = (0u8..=255).cycle().take(200_000).collect::<Vec<u8>>();
        let entries = vec![NamedEntry {
            name: "large.bin".to_string(),
            data: big.clone(),
        }];
        let mut psarc = round_trip(entries, true);

        let data = psarc.inflate_file("large.bin").unwrap();
        assert_eq!(data, big);
    }

    #[test]
    fn test_plain_sng_file_not_compressed() {
        // .sng files must be stored uncompressed.
        let sng_data: Vec<u8> = (0u8..=127).cycle().take(512).collect();
        let entries = vec![NamedEntry {
            name: "song.sng".to_string(),
            data: sng_data.clone(),
        }];
        let mut psarc = round_trip(entries, false);
        assert_eq!(psarc.inflate_file("song.sng").unwrap(), sng_data);
    }

    #[test]
    fn test_plain_wem_with_zlib_like_prefix_not_inflated() {
        let mut encoded = Vec::new();
        {
            let mut z = ZlibEncoder::new(&mut encoded, Compression::best());
            use std::io::Write;
            z.write_all(b"short").unwrap();
            z.finish().unwrap();
        }
        encoded.extend_from_slice(b"__trailing_plain_wem_bytes__");

        let entries = vec![NamedEntry {
            name: "audio/windows/test.wem".to_string(),
            data: encoded.clone(),
        }];
        let mut psarc = round_trip(entries, false);

        assert_eq!(
            psarc.inflate_file("audio/windows/test.wem").unwrap(),
            encoded
        );
    }

    #[test]
    fn test_file_not_found() {
        let mut psarc = round_trip(vec![], false);
        let result = psarc.inflate_file("nonexistent.txt");
        assert!(matches!(result, Err(PsarcError::FileNotFound(_))));
    }

    #[test]
    fn test_inflate_entry() {
        let payload = b"entry data";
        let entries = vec![NamedEntry {
            name: "entry.bin".to_string(),
            data: payload.to_vec(),
        }];
        let mut psarc = round_trip(entries, false);

        let entry = psarc.toc()[0].clone();
        let data = psarc.inflate_entry(&entry).unwrap();
        assert_eq!(data, payload);
    }

    #[test]
    fn test_z_type() {
        assert_eq!(z_type(65536), 2);
        assert_eq!(z_type(16777216), 3);
        // 4294967295 = 2^32 - 1 < 256^4, so z_type = 3
        assert_eq!(z_type(4294967295), 3);
        // The only way to get z_type 4 would be block_size_alloc > 256^3,
        // but since it is stored as u32, the maximum is 2^32 - 1 which gives 3.
        assert_eq!(z_type(1 << 24), 3); // 16777216 = 256^3
    }

    #[test]
    fn test_has_zlib_header() {
        assert!(has_zlib_header(&[0x78, 0xDA, 0x00]));
        assert!(has_zlib_header(&[0x78, 0x9C, 0x00]));
        assert!(has_zlib_header(&[0x78, 0x01, 0x00]));
        assert!(!has_zlib_header(&[0x00, 0xDA]));
        assert!(!has_zlib_header(&[0x78, 0x00]));
        assert!(!has_zlib_header(&[]));
    }
}
