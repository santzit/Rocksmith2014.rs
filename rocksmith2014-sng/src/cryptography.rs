//! AES-256-CTR encryption/decryption for SNG files.

use aes::Aes256;
use ctr::{
    cipher::{KeyIvInit, StreamCipher},
    Ctr128BE,
};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rand::RngCore;
use std::io::{Read, Write};

use crate::{Error, Platform, Result};

pub(crate) type AesCtr = Ctr128BE<Aes256>;

pub(crate) const PC_KEY: [u8; 32] = [
    0xCB, 0x64, 0x8D, 0xF3, 0xD1, 0x2A, 0x16, 0xBF, 0x71, 0x70, 0x14, 0x14, 0xE6, 0x96, 0x19, 0xEC,
    0x17, 0x1C, 0xCA, 0x5D, 0x2A, 0x14, 0x2E, 0x3E, 0x59, 0xDE, 0x7A, 0xDD, 0xA1, 0x8A, 0x3A, 0x30,
];
pub(crate) const MAC_KEY: [u8; 32] = [
    0x98, 0x21, 0x33, 0x0E, 0x34, 0xB9, 0x1F, 0x70, 0xD0, 0xA4, 0x8C, 0xBD, 0x62, 0x59, 0x93, 0x12,
    0x69, 0x70, 0xCE, 0xA0, 0x91, 0x92, 0xC0, 0xE6, 0xCD, 0xA6, 0x76, 0xCC, 0x98, 0x38, 0x28, 0x9D,
];

pub fn decrypt_sng(input: &[u8], platform: Platform) -> Result<Vec<u8>> {
    if input.len() < 24 || input[0..4] != [0x4A, 0, 0, 0] {
        return Err(Error::InvalidHeader);
    }
    let iv = &input[8..24];
    let key = match platform {
        Platform::Pc => &PC_KEY,
        Platform::Mac => &MAC_KEY,
    };
    let mut payload = input[24..].to_vec();
    let mut cipher = AesCtr::new_from_slices(key, iv).map_err(|_| Error::Crypto)?;
    cipher.apply_keystream(&mut payload);

    if payload.len() < 4 {
        return Err(Error::InvalidHeader);
    }
    let compressed = &payload[4..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

pub fn encrypt_sng(data: &[u8], platform: Platform) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;

    let mut payload = Vec::with_capacity(4 + compressed.len());
    payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
    payload.extend_from_slice(&compressed);

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    let key = match platform {
        Platform::Pc => &PC_KEY,
        Platform::Mac => &MAC_KEY,
    };
    let mut cipher = AesCtr::new_from_slices(key, &iv).map_err(|_| Error::Crypto)?;
    cipher.apply_keystream(&mut payload);

    let mut out = Vec::new();
    out.extend_from_slice(&[0x4A, 0, 0, 0]);
    out.extend_from_slice(&[3, 0, 0, 0]);
    out.extend_from_slice(&iv);
    out.extend_from_slice(&payload);
    out.extend_from_slice(&[0u8; 56]);
    Ok(out)
}
