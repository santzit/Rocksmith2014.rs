use std::{
    io::{self, Cursor, Read},
    path::Path,
};

use aes::Aes256;
use cfb_mode::{BufDecryptor, cipher::KeyIvInit};
use flate2::read::ZlibDecoder;
use md5::Digest;

use crate::error::{Error, Result};

/// AES-256 key used to decrypt the PSARC table of contents.
const PSARC_KEY: [u8; 32] = [
    0xC5, 0x3D, 0xB2, 0x38, 0x70, 0xA1, 0xA2, 0xF7,
    0x1C, 0xAE, 0x64, 0x06, 0x1F, 0xDD, 0x0E, 0x11,
    0x57, 0x30, 0x9D, 0xC8, 0x52, 0x04, 0xD4, 0xC5,
    0xBF, 0xDF, 0x25, 0x09, 0x0D, 0xF2, 0x57, 0x2C,
];

/// Zero IV used for PSARC TOC decryption.
const PSARC_IV: [u8; 16] = [0u8; 16];

/// PSARC magic bytes ("PSAR").
const PSARC_MAGIC: [u8; 4] = *b"PSAR";

/// PSARC compression method ("zlib").
const PSARC_COMPRESSION: [u8; 4] = *b"zlib";

// ---------------------------------------------------------------------------
// PSARC header
// ---------------------------------------------------------------------------

/// Parsed PSARC file header (all fields big-endian in the file).
#[derive(Debug)]
struct Header {
    version_major: u16,
    version_minor: u16,
    toc_length: u32,
    toc_entry_size: u32,
    toc_entry_count: u32,
    block_size_alloc: u32,
    archive_flags: u32,
}

impl Header {
    /// Length of the serialised header in bytes.
    const LENGTH: u32 = 32;

    fn read(r: &mut impl Read) -> Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != PSARC_MAGIC {
            return Err(Error::InvalidPsarc(format!(
                "bad magic {:?}",
                String::from_utf8_lossy(&magic)
            )));
        }

        let version_major = read_u16_be(r)?;
        let version_minor = read_u16_be(r)?;

        let mut compression = [0u8; 4];
        r.read_exact(&mut compression)?;
        if compression != PSARC_COMPRESSION {
            return Err(Error::InvalidPsarc(format!(
                "unsupported compression '{}'",
                String::from_utf8_lossy(&compression)
            )));
        }

        Ok(Header {
            version_major,
            version_minor,
            toc_length: read_u32_be(r)?,
            toc_entry_size: read_u32_be(r)?,
            toc_entry_count: read_u32_be(r)?,
            block_size_alloc: read_u32_be(r)?,
            archive_flags: read_u32_be(r)?,
        })
    }

    /// Returns `true` when the table-of-contents is AES-256-CFB encrypted.
    fn is_encrypted(&self) -> bool {
        self.archive_flags == 4
    }
}

// ---------------------------------------------------------------------------
// TOC entry
// ---------------------------------------------------------------------------

/// One entry in the PSARC table of contents.
#[derive(Debug, Clone)]
struct TocEntry {
    /// MD5 hash of the entry name (16 bytes).
    name_digest: [u8; 16],
    /// First z-block index for this entry.
    z_index_begin: u32,
    /// Uncompressed length of the entry data.
    length: u64,
    /// Absolute byte offset of the entry data inside the PSARC file.
    offset: u64,
}

impl TocEntry {
    fn read(r: &mut impl Read) -> Result<Self> {
        let mut name_digest = [0u8; 16];
        r.read_exact(&mut name_digest)?;
        let z_index_begin = read_u32_be(r)?;
        let length = read_u40_be(r)?;
        let offset = read_u40_be(r)?;
        Ok(TocEntry { name_digest, z_index_begin, length, offset })
    }

    /// Returns `true` when this entry is the manifest (all-zero digest).
    fn is_manifest(&self) -> bool {
        self.name_digest == [0u8; 16]
    }
}

// ---------------------------------------------------------------------------
// Public PSARC type
// ---------------------------------------------------------------------------

/// An opened PSARC archive. Entry data is inflated lazily.
pub struct Psarc {
    /// Raw bytes of the PSARC file held in memory.
    data: Vec<u8>,
    /// The parsed table of contents (without the manifest entry).
    entries: Vec<TocEntry>,
    /// Names of the entries in the same order as `entries`.
    names: Vec<String>,
    /// CString representations of each name (kept alive for FFI).
    names_cstr: Vec<std::ffi::CString>,
    /// Size of each compressed/raw block as read from the block-size table.
    block_sizes: Vec<u32>,
    /// Maximum block size (from the header).
    block_size_alloc: u32,
}

impl Psarc {
    /// Opens and parses a PSARC file from `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    /// Parses a PSARC from an in-memory byte slice.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let mut cur = Cursor::new(&data);

        // --- header ---
        let header = Header::read(&mut cur)?;
        let block_size_alloc = header.block_size_alloc;

        // --- TOC (optionally encrypted) ---
        let toc_payload_len =
            header.toc_length as usize - Header::LENGTH as usize;

        let mut toc_bytes = vec![0u8; toc_payload_len];
        cur.read_exact(&mut toc_bytes)?;

        if header.is_encrypted() {
            let mut dec = BufDecryptor::<Aes256>::new(
                &PSARC_KEY.into(),
                &PSARC_IV.into(),
            );
            dec.decrypt(&mut toc_bytes);
        }

        let mut toc_cur = Cursor::new(&toc_bytes);

        // --- parse TOC entries ---
        let entry_count = header.toc_entry_count as usize;
        let mut raw_entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            raw_entries.push(TocEntry::read(&mut toc_cur)?);
        }

        // --- block-size table ---
        let z_type = {
            let log256 = (block_size_alloc as f64).log(256.0).round() as u32;
            log256.max(2)
        };
        let entry_bytes = (header.toc_entry_size * entry_count as u32) as usize;
        let block_table_bytes =
            toc_payload_len.saturating_sub(entry_bytes);
        let block_count = block_table_bytes / z_type as usize;

        let mut block_sizes = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            let bs = match z_type {
                2 => read_u16_be(&mut toc_cur)? as u32,
                3 => read_u24_be(&mut toc_cur)?,
                4 => read_u32_be(&mut toc_cur)?,
                _ => return Err(Error::InvalidPsarc("unexpected z_type".into())),
            };
            block_sizes.push(bs);
        }

        // --- manifest (first entry when count > 1) ---
        // Real Rocksmith PSARCs have a non-zero manifest digest; the .NET implementation
        // unconditionally treats entry 0 as the manifest regardless of its digest.
        let (manifest_entry, data_entries) = if raw_entries.len() > 1 {
            let mut it = raw_entries.into_iter();
            let manifest = it.next().unwrap();
            (Some(manifest), it.collect::<Vec<_>>())
        } else {
            (None, raw_entries)
        };

        let names: Vec<String> = if let Some(m) = manifest_entry {
            let manifest_data =
                inflate_entry(&data, &m, &block_sizes, block_size_alloc)?;
            let text = String::from_utf8_lossy(&manifest_data).into_owned();
            text.lines().map(str::to_owned).collect()
        } else {
            // Reconstruct names from MD5 if no manifest present
            data_entries
                .iter()
                .map(|e| format!("{}", hex_digest(&e.name_digest)))
                .collect()
        };

        let names_cstr = names
            .iter()
            .map(|n| std::ffi::CString::new(n.as_bytes()).unwrap_or_default())
            .collect();

        Ok(Psarc {
            data,
            entries: data_entries,
            names,
            names_cstr,
            block_sizes,
            block_size_alloc,
        })
    }

    /// Returns the number of entries in this archive.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the name of the entry at `index`.
    pub fn entry_name(&self, index: usize) -> Result<&str> {
        self.names
            .get(index)
            .map(String::as_str)
            .ok_or(Error::IndexOutOfRange(index, self.entries.len()))
    }

    /// Returns all entry names.
    pub fn entry_names(&self) -> &[String] {
        &self.names
    }

    /// Inflates and returns the raw bytes of the entry at `index`.
    pub fn extract(&self, index: usize) -> Result<Vec<u8>> {
        let entry = self
            .entries
            .get(index)
            .ok_or(Error::IndexOutOfRange(index, self.entries.len()))?;
        inflate_entry(&self.data, entry, &self.block_sizes, self.block_size_alloc)
    }

    /// Looks up the first entry whose name ends with `suffix` (case-insensitive).
    pub fn find_by_suffix(&self, suffix: &str) -> Option<usize> {
        let suffix_lower = suffix.to_lowercase();
        self.names
            .iter()
            .position(|n| n.to_lowercase().ends_with(&suffix_lower))
    }

    // Internal helper for FFI: returns the null-terminated name pointer.
    pub(crate) fn name_ptr(&self, index: usize) -> *const std::os::raw::c_char {
        self.names_cstr
            .get(index)
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    }
}

// ---------------------------------------------------------------------------
// Block inflation
// ---------------------------------------------------------------------------

/// Inflate (decompress) a single PSARC entry from `file_data`.
fn inflate_entry(
    file_data: &[u8],
    entry: &TocEntry,
    block_sizes: &[u32],
    block_size_alloc: u32,
) -> Result<Vec<u8>> {
    let block_size = block_size_alloc as usize;
    let mut output = Vec::with_capacity(entry.length as usize);
    let mut z_index = entry.z_index_begin as usize;
    let mut pos = entry.offset as usize;

    while (output.len() as u64) < entry.length {
        let bs = block_sizes.get(z_index).copied().unwrap_or(0);
        let read_len = if bs == 0 { block_size } else { bs as usize };

        let end = pos.checked_add(read_len).ok_or_else(|| {
            Error::InvalidPsarc("block offset overflow".into())
        })?;
        if end > file_data.len() {
            return Err(Error::InvalidPsarc(format!(
                "block at offset {pos} len {read_len} exceeds file size {}",
                file_data.len()
            )));
        }
        let block = &file_data[pos..end];

        if bs == 0 {
            // Raw, full cluster
            output.extend_from_slice(block);
        } else if has_zlib_header(block) {
            // Zlib-compressed block
            let mut dec = ZlibDecoder::new(block);
            dec.read_to_end(&mut output).map_err(|e| {
                Error::InvalidPsarc(format!("zlib decompress: {e}"))
            })?;
        } else {
            // Stored raw
            output.extend_from_slice(block);
        }

        pos += read_len;
        z_index += 1;
    }

    // Truncate to exact entry length in case a full block was over-read
    output.truncate(entry.length as usize);
    Ok(output)
}

/// Returns `true` when `data` starts with the zlib best-compression header.
#[inline]
fn has_zlib_header(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x78 && data[1] == 0xDA
}

// ---------------------------------------------------------------------------
// Binary reading helpers (big-endian)
// ---------------------------------------------------------------------------

fn read_u16_be(r: &mut impl Read) -> io::Result<u16> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_be_bytes(b))
}

fn read_u32_be(r: &mut impl Read) -> io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

fn read_u24_be(r: &mut impl Read) -> io::Result<u32> {
    let mut b = [0u8; 3];
    r.read_exact(&mut b)?;
    Ok((b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32)
}

/// Read a 5-byte big-endian unsigned integer (uint40).
fn read_u40_be(r: &mut impl Read) -> io::Result<u64> {
    let mut b = [0u8; 5];
    r.read_exact(&mut b)?;
    Ok(
        (b[0] as u64) << 32
            | (b[1] as u64) << 24
            | (b[2] as u64) << 16
            | (b[3] as u64) << 8
            | b[4] as u64,
    )
}

/// Format a 16-byte digest as a lowercase hex string.
fn hex_digest(d: &[u8; 16]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// PSARC writer (used only by test-fixture generator)
// ---------------------------------------------------------------------------

/// Minimal PSARC builder for generating test fixtures.
///
/// Produces an **unencrypted** PSARC with one file entry.
pub struct PsarcBuilder {
    entries: Vec<(String, Vec<u8>)>,
}

impl PsarcBuilder {
    pub fn new() -> Self {
        PsarcBuilder { entries: Vec::new() }
    }

    /// Add a named entry with the given raw bytes (stored uncompressed).
    pub fn add_entry(&mut self, name: impl Into<String>, data: Vec<u8>) {
        self.entries.push((name.into(), data));
    }

    /// Serialise to PSARC bytes.
    pub fn build(self) -> Vec<u8> {
        const BLOCK_SIZE: u32 = 65536;
        const ENTRY_SIZE: u32 = 30;

        let entry_count = self.entries.len() + 1; // +1 for manifest

        // Manifest data: names joined by '\n'
        let manifest_data: Vec<u8> = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(i, (name, _))| {
                let mut v = Vec::new();
                if i != 0 {
                    v.push(b'\n');
                }
                v.extend_from_slice(name.as_bytes());
                v
            })
            .collect();

        // Number of blocks = number of entries (one block each, all < 64KB)
        let block_count = entry_count; // 1 block per entry
        let z_type = 2u32; // 2 bytes per entry (log256(65536) = 2)

        let toc_entries_len = entry_count as u32 * ENTRY_SIZE;
        let block_table_len = block_count as u32 * z_type;
        let toc_payload_len = toc_entries_len + block_table_len;
        let toc_total_len = 32 + toc_payload_len; // header + toc

        // Calculate data offsets
        let base_offset = toc_total_len as u64;

        let manifest_offset = base_offset;
        let manifest_len = manifest_data.len() as u64;

        let mut data_offsets = Vec::with_capacity(self.entries.len());
        let mut cur_off = manifest_offset + manifest_len;
        for (_, data) in &self.entries {
            data_offsets.push(cur_off);
            cur_off += data.len() as u64;
        }

        let mut out = Vec::new();

        // --- Header ---
        out.extend_from_slice(b"PSAR");
        out.extend_from_slice(&1u16.to_be_bytes()); // version major
        out.extend_from_slice(&4u16.to_be_bytes()); // version minor
        out.extend_from_slice(b"zlib");
        out.extend_from_slice(&toc_total_len.to_be_bytes()); // ToCLength
        out.extend_from_slice(&ENTRY_SIZE.to_be_bytes()); // ToCEntrySize
        out.extend_from_slice(&(entry_count as u32).to_be_bytes()); // ToCEntryCount
        out.extend_from_slice(&BLOCK_SIZE.to_be_bytes()); // BlockSizeAlloc
        out.extend_from_slice(&0u32.to_be_bytes()); // ArchiveFlags (0 = no encryption)

        // --- Manifest entry ---
        out.extend_from_slice(&[0u8; 16]); // zero digest
        out.extend_from_slice(&0u32.to_be_bytes()); // z_index_begin = 0
        write_u40_be(&mut out, manifest_len);
        write_u40_be(&mut out, manifest_offset);

        // --- Data entries ---
        for (i, (name, data)) in self.entries.iter().enumerate() {
            let digest = md5::Md5::digest(name.as_bytes());
            out.extend_from_slice(&digest);
            out.extend_from_slice(&((i + 1) as u32).to_be_bytes()); // z_index_begin
            write_u40_be(&mut out, data.len() as u64);
            write_u40_be(&mut out, data_offsets[i]);
        }

        // --- Block-size table (2 bytes per block) ---
        // Manifest block
        out.extend_from_slice(&(manifest_len as u16).to_be_bytes());
        // Data blocks
        for (_, data) in &self.entries {
            out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        }

        // --- Payload ---
        out.extend_from_slice(&manifest_data);
        for (_, data) in &self.entries {
            out.extend_from_slice(data);
        }

        out
    }
}

fn write_u40_be(out: &mut Vec<u8>, v: u64) {
    out.push(((v >> 32) & 0xFF) as u8);
    out.push(((v >> 24) & 0xFF) as u8);
    out.push(((v >> 16) & 0xFF) as u8);
    out.push(((v >> 8) & 0xFF) as u8);
    out.push((v & 0xFF) as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_single_entry() {
        let xml = b"<song><title>Hello</title></song>".to_vec();
        let mut builder = PsarcBuilder::new();
        builder.add_entry("songs/arr/hello.xml", xml.clone());
        let psarc_bytes = builder.build();

        let psarc = Psarc::from_bytes(psarc_bytes).unwrap();
        assert_eq!(psarc.entry_count(), 1);
        assert_eq!(psarc.entry_name(0).unwrap(), "songs/arr/hello.xml");
        assert_eq!(psarc.extract(0).unwrap(), xml);
    }

    #[test]
    fn roundtrip_multiple_entries() {
        let xml1 = b"<a/>".to_vec();
        let xml2 = b"<b/>".to_vec();
        let mut builder = PsarcBuilder::new();
        builder.add_entry("file1.xml", xml1.clone());
        builder.add_entry("file2.xml", xml2.clone());
        let psarc_bytes = builder.build();

        let psarc = Psarc::from_bytes(psarc_bytes).unwrap();
        assert_eq!(psarc.entry_count(), 2);
        assert_eq!(psarc.entry_name(0).unwrap(), "file1.xml");
        assert_eq!(psarc.entry_name(1).unwrap(), "file2.xml");
        assert_eq!(psarc.extract(0).unwrap(), xml1);
        assert_eq!(psarc.extract(1).unwrap(), xml2);
    }
}
