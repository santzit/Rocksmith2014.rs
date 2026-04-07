use std::io::{self, Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

/// Compresses data from `input` into `output` using zlib.
pub fn zip(input: &mut dyn Read, output: &mut dyn Write) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(output, Compression::best());
    io::copy(input, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Decompresses zlib data from `input` into `output`.
pub fn unzip(input: &mut dyn Read, output: &mut dyn Write) -> io::Result<()> {
    let mut decoder = ZlibDecoder::new(input);
    io::copy(&mut decoder, output)?;
    Ok(())
}

/// Compresses `data` bytes and returns the compressed bytes.
pub fn zip_bytes(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    zip(&mut &*data, &mut out)?;
    Ok(out)
}

/// Decompresses `data` bytes and returns the decompressed bytes.
pub fn unzip_bytes(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    unzip(&mut &*data, &mut out)?;
    Ok(out)
}
