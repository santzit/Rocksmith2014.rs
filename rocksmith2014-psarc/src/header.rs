use std::io::{self, Read, Write};

use crate::error::{PsarcError, Result};

/// The size of a PSARC header in bytes.
pub(crate) const HEADER_SIZE: usize = 32;

const MAGIC: &[u8; 4] = b"PSAR";
const COMPRESSION_METHOD: &[u8; 4] = b"zlib";

/// The PSARC archive header (32 bytes, big-endian).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Major version number (should be 1).
    pub version_major: u16,
    /// Minor version number (should be 4).
    pub version_minor: u16,
    /// Length of the TOC in bytes, including the header (32 bytes) and the block size table.
    pub toc_length: u32,
    /// Size of each TOC entry in bytes (default: 30).
    pub toc_entry_size: u32,
    /// Number of entries in the table of contents.
    pub toc_entry_count: u32,
    /// Maximum size of a block in bytes (default: 65536).
    pub block_size_alloc: u32,
    /// Configuration flags for the archive (bit 2 = TOC encrypted).
    pub archive_flags: u32,
}

impl Header {
    /// Creates a new header with default values.
    pub fn new() -> Self {
        Header {
            version_major: 1,
            version_minor: 4,
            toc_length: 0,
            toc_entry_size: 30,
            toc_entry_count: 0,
            block_size_alloc: 65536,
            archive_flags: 0,
        }
    }

    /// Returns `true` if the TOC is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.archive_flags & 4 != 0
    }

    /// Reads a PSARC header from the given reader.
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(PsarcError::InvalidHeader("magic check failed".to_string()));
        }

        let version_major = read_u16_be(reader)?;
        let version_minor = read_u16_be(reader)?;

        let mut comp = [0u8; 4];
        reader.read_exact(&mut comp)?;
        if &comp != COMPRESSION_METHOD {
            return Err(PsarcError::Unsupported(format!(
                "unsupported compression method: {}",
                String::from_utf8_lossy(&comp)
            )));
        }

        Ok(Header {
            version_major,
            version_minor,
            toc_length: read_u32_be(reader)?,
            toc_entry_size: read_u32_be(reader)?,
            toc_entry_count: read_u32_be(reader)?,
            block_size_alloc: read_u32_be(reader)?,
            archive_flags: read_u32_be(reader)?,
        })
    }

    /// Writes this header to the given writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(&self.version_major.to_be_bytes())?;
        writer.write_all(&self.version_minor.to_be_bytes())?;
        writer.write_all(COMPRESSION_METHOD)?;
        writer.write_all(&self.toc_length.to_be_bytes())?;
        writer.write_all(&self.toc_entry_size.to_be_bytes())?;
        writer.write_all(&self.toc_entry_count.to_be_bytes())?;
        writer.write_all(&self.block_size_alloc.to_be_bytes())?;
        writer.write_all(&self.archive_flags.to_be_bytes())?;
        Ok(())
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

fn read_u16_be<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32_be<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_roundtrip() {
        let header = Header {
            version_major: 1,
            version_minor: 4,
            toc_length: 1234,
            toc_entry_size: 30,
            toc_entry_count: 5,
            block_size_alloc: 65536,
            archive_flags: 4,
        };

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();
        assert_eq!(buf.len(), HEADER_SIZE);

        let parsed = Header::read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(parsed, header);
    }

    #[test]
    fn test_header_invalid_magic() {
        let bad: Vec<u8> = vec![0u8; 32];
        let result = Header::read(&mut Cursor::new(bad));
        assert!(matches!(result, Err(PsarcError::InvalidHeader(_))));
    }

    #[test]
    fn test_is_encrypted() {
        let mut h = Header::new();
        assert!(!h.is_encrypted());
        h.archive_flags = 4;
        assert!(h.is_encrypted());
    }
}
