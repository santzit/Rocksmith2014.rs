use std::io::{self, Read, Write};

/// A single entry in the PSARC table of contents (30 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// MD5 hash of the entry's name as listed in the manifest.
    pub name_digest: [u8; 16],
    /// Index of the first block in the block size table for this entry.
    pub z_index_begin: u32,
    /// Uncompressed length of the entry's data in bytes (40-bit).
    pub length: u64,
    /// Byte offset from the start of the archive file (40-bit).
    pub offset: u64,
    /// Zero-based index of this entry (not stored on disk; derived from read order).
    pub id: usize,
}

impl Entry {
    /// Reads an entry from the given reader.
    pub fn read<R: Read>(reader: &mut R, id: usize) -> io::Result<Self> {
        let mut digest = [0u8; 16];
        reader.read_exact(&mut digest)?;

        let z_index_begin = read_u32_be(reader)?;
        let length = read_u40_be(reader)?;
        let offset = read_u40_be(reader)?;

        Ok(Entry {
            name_digest: digest,
            z_index_begin,
            length,
            offset,
            id,
        })
    }

    /// Writes this entry to the given writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.name_digest)?;
        writer.write_all(&self.z_index_begin.to_be_bytes())?;
        write_u40_be(writer, self.length)?;
        write_u40_be(writer, self.offset)?;
        Ok(())
    }
}

/// Reads a 4-byte big-endian unsigned integer.
fn read_u32_be<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Reads a 5-byte (40-bit) big-endian unsigned integer into a `u64`.
fn read_u40_be<R: Read>(reader: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 5];
    reader.read_exact(&mut buf)?;
    Ok(((buf[0] as u64) << 32)
        | ((buf[1] as u64) << 24)
        | ((buf[2] as u64) << 16)
        | ((buf[3] as u64) << 8)
        | (buf[4] as u64))
}

/// Writes a `u64` value as a 5-byte (40-bit) big-endian integer.
fn write_u40_be<W: Write>(writer: &mut W, value: u64) -> io::Result<()> {
    writer.write_all(&[
        ((value >> 32) & 0xFF) as u8,
        ((value >> 24) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_entry_roundtrip() {
        let entry = Entry {
            name_digest: [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0x10,
            ],
            z_index_begin: 42,
            length: 0x0102030405,
            offset: 0x0A0B0C0D0E,
            id: 3,
        };

        let mut buf = Vec::new();
        entry.write(&mut buf).unwrap();
        assert_eq!(buf.len(), 30);

        let parsed = Entry::read(&mut Cursor::new(&buf), 3).unwrap();
        assert_eq!(parsed, entry);
    }

    #[test]
    fn test_u40_max() {
        let max_val = 0xFF_FFFF_FFFF_u64;
        let mut buf = Vec::new();
        write_u40_be(&mut buf, max_val).unwrap();
        let result = read_u40_be(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(result, max_val);
    }
}
