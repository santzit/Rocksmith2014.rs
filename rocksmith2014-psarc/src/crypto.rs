use aes::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Aes256,
};
use md5::{Digest, Md5};

/// The 256-bit AES key used to encrypt/decrypt the PSARC table of contents.
const PSARC_KEY: [u8; 32] = [
    0xC5, 0x3D, 0xB2, 0x38, 0x70, 0xA1, 0xA2, 0xF7, 0x1C, 0xAE, 0x64, 0x06, 0x1F, 0xDD, 0x0E,
    0x11, 0x57, 0x30, 0x9D, 0xC8, 0x52, 0x04, 0xD4, 0xC5, 0xBF, 0xDF, 0x25, 0x09, 0x0D, 0xF2,
    0x57, 0x2C,
];

const BLOCK_SIZE: usize = 16;

/// Decrypts `data` in-place using AES-256 in CFB-128 mode with a zero IV.
///
/// CFB-128 encryption: `C_i = E(C_{i-1}) XOR P_i`, where `C_0 = IV`.
/// Decryption: `P_i = E(C_{i-1}) XOR C_i`, where `C_0 = IV`.
/// Both operations use `BlockEncrypt` (never decrypt) because CFB feeds ciphertext back.
pub(crate) fn decrypt(ciphertext: &[u8]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(&PSARC_KEY));
    let mut result = ciphertext.to_vec();
    let mut iv = [0u8; BLOCK_SIZE];

    let mut i = 0;
    while i < result.len() {
        let chunk_end = (i + BLOCK_SIZE).min(result.len());

        // Encrypt the previous ciphertext block (IV for first block) to get keystream.
        let mut keystream = GenericArray::from(iv);
        cipher.encrypt_block(&mut keystream);

        // Save the ciphertext bytes before XOR (needed as next IV).
        let mut ct_block = [0u8; BLOCK_SIZE];
        ct_block[..chunk_end - i].copy_from_slice(&result[i..chunk_end]);

        // XOR keystream with ciphertext to recover plaintext.
        for (b, k) in result[i..chunk_end].iter_mut().zip(keystream.iter()) {
            *b ^= k;
        }

        // The next IV is the original (encrypted) ciphertext block.
        iv = ct_block;
        i += BLOCK_SIZE;
    }

    result
}

/// Encrypts `plaintext` using AES-256 in CFB-128 mode with a zero IV.
///
/// CFB-128 encryption: `C_i = E(C_{i-1}) XOR P_i`, where `C_0 = IV`.
pub(crate) fn encrypt(plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(&PSARC_KEY));
    let mut result = plaintext.to_vec();
    let mut iv = [0u8; BLOCK_SIZE];

    let mut i = 0;
    while i < result.len() {
        let chunk_end = (i + BLOCK_SIZE).min(result.len());

        // Encrypt the previous ciphertext block (IV for first block) to get keystream.
        let mut keystream = GenericArray::from(iv);
        cipher.encrypt_block(&mut keystream);

        // XOR keystream with plaintext to produce ciphertext.
        for (b, k) in result[i..chunk_end].iter_mut().zip(keystream.iter()) {
            *b ^= k;
        }

        // The next IV is the ciphertext we just produced.
        iv = [0u8; BLOCK_SIZE];
        iv[..chunk_end - i].copy_from_slice(&result[i..chunk_end]);
        i += BLOCK_SIZE;
    }

    result
}

/// Calculates the MD5 hash of an ASCII-encoded file name.
///
/// Returns all-zero bytes for an empty name (used for the manifest entry).
pub(crate) fn md5_hash(name: &str) -> [u8; 16] {
    if name.is_empty() {
        [0u8; 16]
    } else {
        Md5::digest(name.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, Rocksmith!";
        let encrypted = encrypt(plaintext);
        let decrypted = decrypt(&encrypted);
        assert_eq!(&decrypted[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_full_blocks() {
        let plaintext = vec![0x42u8; 64]; // 4 full AES blocks
        let encrypted = encrypt(&plaintext);
        assert_ne!(encrypted, plaintext);
        let decrypted = decrypt(&encrypted);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_md5_hash_empty() {
        let hash = md5_hash("");
        assert_eq!(hash, [0u8; 16]);
    }

    #[test]
    fn test_md5_hash_known_value() {
        // MD5("abc") = 900150983cd24fb0d6963f7d28e17f72
        let hash = md5_hash("abc");
        assert_eq!(
            hash,
            [
                0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28,
                0xe1, 0x7f, 0x72
            ]
        );
    }
}
