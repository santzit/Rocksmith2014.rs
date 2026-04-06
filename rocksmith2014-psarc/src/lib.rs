//! Rust implementation of the Rocksmith 2014 PSARC archive format.
//!
//! PSARC is a proprietary archive format used by Rocksmith 2014 to package
//! downloadable content (DLC). Archives contain a manifest listing all file
//! paths and a table of contents (TOC) with per-entry metadata. The TOC may
//! optionally be encrypted with AES-256 in CFB-128 mode. Individual file data
//! is split into 64 KB blocks and compressed with zlib.
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

mod crypto;
mod entry;
mod error;
mod header;

pub use entry::Entry;
pub use error::{PsarcError, Result};
pub use header::Header;

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use header::HEADER_SIZE;
use std::{
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

/// A named file entry to be packed into a PSARC archive.
#[derive(Debug, Clone)]
pub struct NamedEntry {
    /// Path of the file as it will appear in the archive manifest.
    pub name: String,
    /// Raw file contents.
    pub data: Vec<u8>,
}

/// A Rocksmith 2014 PSARC archive.
pub struct Psarc<R> {
    source: R,
    header: Header,
    toc: Vec<Entry>,
    block_sizes: Vec<u32>,
    manifest: Vec<String>,
}

impl<R: Read + Seek> Psarc<R> {
    /// Opens a PSARC archive from the given reader.
    ///
    /// Parses the header, optionally decrypts the TOC, reads the block size
    /// table, and loads the manifest.
    pub fn read(mut reader: R) -> Result<Self> {
        let header = Header::read(&mut reader)?;

        let toc_payload_size = header.toc_length as usize - HEADER_SIZE;
        let mut toc_raw = vec![0u8; toc_payload_size];
        reader.read_exact(&mut toc_raw)?;

        let toc_bytes = if header.is_encrypted() {
            let decrypted = crypto::decrypt(&toc_raw);
            if decrypted.len() < toc_payload_size {
                return Err(PsarcError::DecryptionFailed);
            }
            decrypted[..toc_payload_size].to_vec()
        } else {
            toc_raw
        };

        let (toc, block_sizes) = parse_toc(&header, &toc_bytes)?;

        let mut psarc = Psarc {
            source: reader,
            header,
            toc,
            block_sizes,
            manifest: Vec::new(),
        };

        // The first TOC entry is always the manifest.
        if !psarc.toc.is_empty() {
            let manifest_entry = psarc.toc[0].clone();
            let raw = psarc.read_blocks(&manifest_entry)?;
            psarc.manifest = parse_manifest(&raw);
            psarc.toc.remove(0);
        }

        Ok(psarc)
    }

    /// Returns the list of file paths stored in the archive.
    pub fn manifest(&self) -> &[String] {
        &self.manifest
    }

    /// Returns the table of contents entries (excluding the manifest entry).
    pub fn toc(&self) -> &[Entry] {
        &self.toc
    }

    /// Inflates the given TOC entry and returns its decompressed bytes.
    pub fn inflate_entry(&mut self, entry: &Entry) -> Result<Vec<u8>> {
        self.read_blocks(entry)
    }

    /// Inflates the file with the given name and returns its decompressed bytes.
    pub fn inflate_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .manifest
            .iter()
            .position(|n| n == name)
            .map(|i| self.toc[i].clone())
            .ok_or_else(|| PsarcError::FileNotFound(name.to_string()))?;
        self.read_blocks(&entry)
    }

    /// Extracts all files from the archive into `base_dir`.
    ///
    /// Subdirectories are created as needed. File paths use the separators of
    /// the host operating system.
    pub fn extract_all(&mut self, base_dir: &Path) -> Result<()> {
        let toc = self.toc.clone();
        let manifest = self.manifest.clone();

        for (i, entry) in toc.iter().enumerate() {
            let rel = manifest[i].replace('/', std::path::MAIN_SEPARATOR_STR);
            let dest = base_dir.join(&rel);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let data = self.read_blocks(entry)?;
            std::fs::write(&dest, data)?;
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    /// Reads and decompresses all blocks belonging to `entry`.
    fn read_blocks(&mut self, entry: &Entry) -> Result<Vec<u8>> {
        let block_size = self.header.block_size_alloc as usize;
        let mut z_index = entry.z_index_begin as usize;
        let mut result = Vec::with_capacity(entry.length as usize);

        self.source.seek(SeekFrom::Start(entry.offset))?;

        while (result.len() as u64) < entry.length {
            let remaining = (entry.length - result.len() as u64) as usize;
            let compressed_size = self.block_sizes[z_index] as usize;

            if compressed_size == 0 {
                // Raw full block; never exceeds `entry.length` because we stop early.
                let to_read = block_size.min(remaining);
                let mut buf = vec![0u8; to_read];
                self.source.read_exact(&mut buf)?;
                result.extend_from_slice(&buf);
            } else {
                let mut buf = vec![0u8; compressed_size];
                self.source.read_exact(&mut buf)?;

                if has_zlib_header(&buf) {
                    let mut decoder = ZlibDecoder::new(Cursor::new(&buf));
                    let mut decompressed = Vec::new();
                    if decoder.read_to_end(&mut decompressed).is_ok() {
                        result.extend_from_slice(&decompressed);
                    } else {
                        // Edge case: a .wem block may start with zlib-like bytes
                        // but is not actually compressed.
                        result.extend_from_slice(&buf);
                    }
                } else {
                    result.extend_from_slice(&buf);
                }
            }

            z_index += 1;
        }

        Ok(result)
    }
}

impl Psarc<std::fs::File> {
    /// Opens a PSARC archive from the file at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::read(file)
    }
}

// =============================================================================
// Writing / Creation
// =============================================================================

impl Psarc<()> {
    /// Creates a new PSARC archive and writes it to `writer`.
    ///
    /// `encrypt` controls whether the TOC is encrypted with AES-256 CFB-128.
    /// `entries` is the list of named files to include.
    pub fn create<W: Write>(mut writer: W, encrypt: bool, entries: Vec<NamedEntry>) -> Result<()> {
        let mut header = Header::new();
        if encrypt {
            header.archive_flags = 4;
        }

        let block_size = header.block_size_alloc as usize;
        let z_type = z_type(header.block_size_alloc);

        // Build manifest text (newline-separated list of entry names).
        let manifest_text = entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if i == 0 {
                    e.name.clone()
                } else {
                    format!("\n{}", e.name)
                }
            })
            .collect::<String>();
        let manifest_bytes = manifest_text.into_bytes();

        // All file slots: manifest first, then actual entries.
        // The manifest has an empty name (used for MD5 = [0; 16]).
        let all: Vec<(&str, &[u8])> = std::iter::once(("", manifest_bytes.as_slice()))
            .chain(entries.iter().map(|e| (e.name.as_str(), e.data.as_slice())))
            .collect();

        // Compress data and build the block size table.
        let mut compressed_blocks: Vec<Vec<u8>> = Vec::new();
        let mut raw_block_sizes: Vec<u32> = Vec::new();
        // (name_digest, z_index_begin, uncompressed_length, total_compressed_size)
        let mut proto: Vec<([u8; 16], u32, u64, u64)> = Vec::new();

        for (name, data) in &all {
            let z_begin = raw_block_sizes.len() as u32;
            let mut total_compressed: u64 = 0;

            if use_plain(name) {
                // Store plain (uncompressed) in chunks.
                if data.is_empty() {
                    raw_block_sizes.push(0);
                    compressed_blocks.push(Vec::new());
                } else {
                    for chunk in data.chunks(block_size) {
                        let sz = chunk.len() as u32;
                        raw_block_sizes.push(sz);
                        compressed_blocks.push(chunk.to_vec());
                        total_compressed += sz as u64;
                    }
                }
            } else {
                // Compress each block independently.
                if data.is_empty() {
                    raw_block_sizes.push(0);
                    compressed_blocks.push(Vec::new());
                } else {
                    for chunk in data.chunks(block_size) {
                        let compressed = compress_block(chunk)?;
                        let compressed_len = compressed.len();
                        let (stored, sz) = if compressed_len < chunk.len() {
                            (compressed, compressed_len as u32)
                        } else {
                            (chunk.to_vec(), chunk.len() as u32)
                        };
                        raw_block_sizes.push(sz);
                        compressed_blocks.push(stored);
                        total_compressed += sz as u64;
                    }
                }
            }

            let digest = crypto::md5_hash(name);
            proto.push((digest, z_begin, data.len() as u64, total_compressed));
        }

        // Convert block sizes: full blocks (== block_size_alloc) become 0.
        let stored_block_sizes: Vec<u32> = raw_block_sizes
            .iter()
            .map(|&s| {
                if s == header.block_size_alloc {
                    0
                } else {
                    s
                }
            })
            .collect();

        // Calculate header fields.
        let toc_entries_size = all.len() * 30;
        let block_table_size = stored_block_sizes.len() * z_type;
        header.toc_length = (HEADER_SIZE + toc_entries_size + block_table_size) as u32;
        header.toc_entry_count = all.len() as u32;

        // Assign offsets.
        let mut offset = header.toc_length as u64;
        let entries_with_offset: Vec<_> = proto
            .iter()
            .map(|(digest, z_begin, length, compressed_size)| {
                let entry_offset = offset;
                offset += compressed_size;
                Entry {
                    name_digest: *digest,
                    z_index_begin: *z_begin,
                    length: *length,
                    offset: entry_offset,
                    id: 0,
                }
            })
            .collect();

        // Serialize TOC.
        let mut toc_buf: Vec<u8> =
            Vec::with_capacity(toc_entries_size + block_table_size);
        for entry in &entries_with_offset {
            entry.write(&mut toc_buf)?;
        }

        // Write block size table.
        for &sz in &stored_block_sizes {
            match z_type {
                2 => toc_buf.extend_from_slice(&(sz as u16).to_be_bytes()),
                3 => {
                    toc_buf.push(((sz >> 16) & 0xFF) as u8);
                    toc_buf.push(((sz >> 8) & 0xFF) as u8);
                    toc_buf.push((sz & 0xFF) as u8);
                }
                4 => toc_buf.extend_from_slice(&sz.to_be_bytes()),
                _ => unreachable!("unexpected z_type"),
            }
        }

        // Write header.
        header.write(&mut writer)?;

        // Write TOC (optionally encrypted).
        if encrypt {
            let encrypted = crypto::encrypt(&toc_buf);
            // The encrypted data may be padded; write only the unpadded length.
            let toc_payload_size = header.toc_length as usize - HEADER_SIZE;
            writer.write_all(&encrypted[..toc_payload_size])?;
        } else {
            writer.write_all(&toc_buf)?;
        }

        // Write compressed file data.
        for block in &compressed_blocks {
            writer.write_all(block)?;
        }

        Ok(())
    }

    /// Packs all files under `dir` (recursively) into a PSARC archive at `output`.
    pub fn pack_directory(dir: &Path, output: &Path, encrypt: bool) -> Result<()> {
        let mut entries = Vec::new();

        for entry in walkdir(dir)? {
            let rel = entry
                .strip_prefix(dir)
                .expect("walkdir returned path outside base")
                .to_string_lossy()
                .replace('\\', "/");
            let data = std::fs::read(&entry)?;
            entries.push(NamedEntry { name: rel, data });
        }

        let file = std::fs::File::create(output)?;
        Self::create(file, encrypt, entries)
    }
}

// =============================================================================
// Private helpers
// =============================================================================

/// Compresses `data` with zlib best-compression and returns the result.
fn compress_block(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(data)?;
    enc.finish()
}

/// Returns `true` if `data` begins with a recognised zlib header byte sequence.
///
/// A zlib stream always starts with `0x78` followed by a byte whose value
/// satisfies `(0x7800 | byte) % 31 == 0`. We check for the three most common
/// variants:
/// - `0x78 0x01` – no compression
/// - `0x78 0x9C` – default compression
/// - `0x78 0xDA` – best compression (used by Rocksmith 2014)
fn has_zlib_header(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }
    data[0] == 0x78 && matches!(data[1], 0x01 | 0x9C | 0xDA)
}

/// Returns `true` if `name` should be stored without compression.
///
/// Matches the behaviour of `Utils.usePlain` in Rocksmith2014.NET:
/// - `.wem` – packed Vorbis audio, already compressed
/// - `.sng` – already zlib-packed
/// - `appid` – very small file
/// - `7z` – already compressed
fn use_plain(name: &str) -> bool {
    name.ends_with(".wem")
        || name.ends_with(".sng")
        || name.ends_with("appid")
        || name.ends_with("7z")
}

/// Returns the number of bytes used per entry in the block size table,
/// equivalent to `floor(log_256(block_size_alloc))`.
///
/// Standard value: `block_size_alloc = 65536` → z_type = 2 (uint16 entries).
pub(crate) fn z_type(block_size_alloc: u32) -> usize {
    let mut x = block_size_alloc as u64;
    let mut result = 0usize;
    while x > 255 {
        x >>= 8;
        result += 1;
    }
    result
}

/// Parses the TOC entries and the block size table from the raw (decrypted)
/// TOC bytes.
fn parse_toc(header: &Header, data: &[u8]) -> Result<(Vec<Entry>, Vec<u32>)> {
    let entry_count = header.toc_entry_count as usize;
    let entry_size = header.toc_entry_size as usize; // always 30
    let toc_bytes = entry_count * entry_size;

    let mut cursor = Cursor::new(data);

    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        let entry = Entry::read(&mut cursor, i)?;
        entries.push(entry);
    }

    let z_type = z_type(header.block_size_alloc);
    let block_table_len = data.len() - toc_bytes;
    let block_count = if z_type > 0 {
        block_table_len / z_type
    } else {
        0
    };

    let mut block_sizes = Vec::with_capacity(block_count);
    for _ in 0..block_count {
        let sz = match z_type {
            2 => {
                let mut b = [0u8; 2];
                cursor.read_exact(&mut b)?;
                u16::from_be_bytes(b) as u32
            }
            3 => {
                let mut b = [0u8; 3];
                cursor.read_exact(&mut b)?;
                ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32
            }
            4 => {
                let mut b = [0u8; 4];
                cursor.read_exact(&mut b)?;
                u32::from_be_bytes(b)
            }
            _ => unreachable!("unexpected z_type"),
        };
        block_sizes.push(sz);
    }

    Ok((entries, block_sizes))
}

/// Splits the manifest bytes into individual file-path strings.
fn parse_manifest(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data);
    text.lines().map(str::to_string).collect()
}

/// Recursively enumerates all regular files under `dir`, sorted for
/// deterministic output.
fn walkdir(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
