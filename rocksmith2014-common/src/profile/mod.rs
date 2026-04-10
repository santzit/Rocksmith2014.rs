//! Rocksmith 2014 profile file decryption/encryption.
//!
//! Mirrors `Profile/Profile.fs` from `Rocksmith2014.Common`.
//!
//! The profile format:
//! - Magic "EVAS" (4 bytes)
//! - Version: u32 LE
//! - Profile ID: u64 LE
//! - Uncompressed length: u32 LE
//! - Body: AES-256-ECB encrypted, then zlib compressed

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;

/// The AES-256-ECB key used for profile encryption.
const PROFILE_KEY: [u8; 32] = [
    0x72, 0x8B, 0x36, 0x9E, 0x24, 0xED, 0x01, 0x34, 0x76, 0x85, 0x11, 0x02, 0x18, 0x12, 0xAF, 0xC0,
    0xA3, 0xC2, 0x5D, 0x02, 0x06, 0x5F, 0x16, 0x6B, 0x4B, 0xCC, 0x58, 0xCD, 0x26, 0x44, 0xF2, 0x9E,
];

/// The profile file magic bytes.
const PROFILE_MAGIC: &[u8; 4] = b"EVAS";

/// Profile header parsed from the file.
#[derive(Debug, Clone)]
pub struct ProfileHeader {
    pub version: u32,
    pub id: u64,
    pub uncompressed_length: u32,
}

/// Error returned when the profile magic bytes are invalid.
#[derive(Debug)]
pub struct ProfileMagicError;

impl std::fmt::Display for ProfileMagicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid profile magic bytes (expected 'EVAS')")
    }
}
impl std::error::Error for ProfileMagicError {}

/// Decrypts AES-256-ECB in-place: each 16-byte block is decrypted independently.
fn decrypt_ecb_inplace(data: &mut [u8]) {
    let key = GenericArray::from_slice(&PROFILE_KEY);
    let cipher = Aes256::new(key);
    for chunk in data.chunks_mut(16) {
        if chunk.len() == 16 {
            let block = GenericArray::from_mut_slice(chunk);
            cipher.decrypt_block(block);
        }
    }
}

/// Encrypts AES-256-ECB in-place: each 16-byte block is encrypted independently.
fn encrypt_ecb_inplace(data: &mut [u8]) {
    let key = GenericArray::from_slice(&PROFILE_KEY);
    let cipher = Aes256::new(key);
    for chunk in data.chunks_mut(16) {
        if chunk.len() == 16 {
            let block = GenericArray::from_mut_slice(chunk);
            cipher.encrypt_block(block);
        }
    }
}

/// Reads and validates the 20-byte profile header.
pub fn read_header(data: &[u8]) -> Result<ProfileHeader, ProfileMagicError> {
    if data.len() < 20 {
        return Err(ProfileMagicError);
    }
    if &data[0..4] != PROFILE_MAGIC {
        return Err(ProfileMagicError);
    }
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let id = u64::from_le_bytes([
        data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
    ]);
    let uncompressed_length = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    Ok(ProfileHeader {
        version,
        id,
        uncompressed_length,
    })
}

/// Decrypts a Rocksmith profile byte slice.
/// Returns the `(header, decompressed_json_bytes)`.
pub fn decrypt(data: &[u8]) -> Result<(ProfileHeader, Vec<u8>), Box<dyn std::error::Error>> {
    let header = read_header(data)?;
    let mut body = data[20..].to_vec();
    decrypt_ecb_inplace(&mut body);
    let mut decompressed = Vec::new();
    crate::compression::unzip(&mut &*body, &mut decompressed)?;
    Ok((header, decompressed))
}

/// Encrypts and wraps profile JSON data into the profile file format.
pub fn encrypt(profile_id: u64, json_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Compress first
    let mut compressed = Vec::new();
    crate::compression::zip(&mut &*json_data, &mut compressed)?;

    // Pad to 16-byte boundary
    let padded_len = (compressed.len() + 15) & !15;
    compressed.resize(padded_len, 0);

    // Encrypt
    encrypt_ecb_inplace(&mut compressed);

    // Build output: magic(4) + version(4) + id(8) + uncompressed_len(4) + body
    let mut out = Vec::with_capacity(20 + compressed.len());
    out.extend_from_slice(PROFILE_MAGIC);
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&profile_id.to_le_bytes());
    out.extend_from_slice(&(json_data.len() as u32).to_le_bytes());
    out.extend_from_slice(&compressed);
    Ok(out)
}
