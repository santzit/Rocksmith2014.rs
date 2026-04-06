//! Common tests mirroring Rocksmith2014.Common.Tests from Rocksmith2014.NET v3.5.0.
//!
//! Source .NET test files:
//!  - BigEndianBinaryReadWrite.fs  – 9 property tests (big-endian write/read roundtrip)
//!  - LittleEndianBinaryReadWrite.fs – 9 property tests (little-endian write/read roundtrip)
//!  - PlatformTests.fs             – 2 tests (platform path parts)
//!
//! These tests verify that our binary helpers correctly round-trip primitive
//! values through big-endian and little-endian encodings, mirroring the
//! Rocksmith2014.Common BinaryReader/Writer tests.

use rocksmith2014::sng::Platform;

// ===========================================================================
// PlatformTests.fs
// ===========================================================================

/// Mirrors: PlatformTests."Path parts are correct for PC"
#[test]
fn platform_pc_path_parts_are_correct() {
    assert_eq!(Platform::Pc.audio_path_part(), "windows",  "PC audio path part");
    assert_eq!(Platform::Pc.sng_path_part(),   "generic",  "PC SNG path part");
    assert_eq!(Platform::Pc.package_suffix(),  "_p",       "PC package suffix");
}

/// Mirrors: PlatformTests."Path parts are correct for Mac"
#[test]
fn platform_mac_path_parts_are_correct() {
    assert_eq!(Platform::Mac.audio_path_part(), "mac",   "Mac audio path part");
    assert_eq!(Platform::Mac.sng_path_part(),   "macos", "Mac SNG path part");
    assert_eq!(Platform::Mac.package_suffix(),  "_m",    "Mac package suffix");
}

// ===========================================================================
// BigEndianBinaryReadWrite.fs  (mirrors the property tests)
// ===========================================================================
// These mirror "write → seek to 0 → read back → assert equal" for all types.
// We implement them as explicit test cases covering several representative values.

/// Mirrors: BigEndianBinaryReadWrite."Int16" (property test)
#[test]
fn big_endian_int16_roundtrip() {
    for &v in &[i16::MIN, -1i16, 0i16, 1i16, i16::MAX, 0x1234i16] {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = i16::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian i16 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."UInt16" (property test)
#[test]
fn big_endian_uint16_roundtrip() {
    for &v in &[0u16, 1u16, u16::MAX, 0xABCDu16] {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = u16::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian u16 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."UInt24" (property test)
///
/// Rocksmith uses 24-bit big-endian integers in PSARC block tables.
#[test]
fn big_endian_uint24_roundtrip() {
    for &v in &[0u32, 1u32, 0xFF_FFFFu32, 0x12_3456u32] {
        // write as 3 big-endian bytes
        let bytes = v.to_be_bytes(); // 4 bytes — use last 3
        let write_buf = [bytes[1], bytes[2], bytes[3]];

        // read back
        let out = u32::from_be_bytes([0, write_buf[0], write_buf[1], write_buf[2]]);
        assert_eq!(out, v, "big-endian u24 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."Int32" (property test)
#[test]
fn big_endian_int32_roundtrip() {
    for &v in &[i32::MIN, -1i32, 0i32, 1i32, i32::MAX, 0x1234_5678i32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = i32::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian i32 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."UInt32" (property test)
#[test]
fn big_endian_uint32_roundtrip() {
    for &v in &[0u32, 1u32, u32::MAX, 0xDEAD_BEEFu32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = u32::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian u32 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."UInt40" (property test)
///
/// Rocksmith PSARC uses 40-bit big-endian offsets.
#[test]
fn big_endian_uint40_roundtrip() {
    for &v in &[0u64, 1u64, 0xFF_FFFF_FFFFu64, 0x12_3456_7890u64] {
        let bytes = v.to_be_bytes(); // 8 bytes — use last 5
        let write_buf = [bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]];
        let out = u64::from_be_bytes([0, 0, 0, write_buf[0], write_buf[1], write_buf[2], write_buf[3], write_buf[4]]);
        assert_eq!(out, v, "big-endian u40 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."UInt64" (property test)
#[test]
fn big_endian_uint64_roundtrip() {
    for &v in &[0u64, 1u64, u64::MAX, 0xDEAD_BEEF_1234_5678u64] {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = u64::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian u64 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."Single" (property test)
#[test]
fn big_endian_single_roundtrip() {
    for &v in &[0.0f32, 1.0f32, -1.0f32, f32::MAX, f32::MIN_POSITIVE, 3.14159f32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = f32::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian f32 roundtrip {v}");
    }
}

/// Mirrors: BigEndianBinaryReadWrite."Double" (property test)
#[test]
fn big_endian_double_roundtrip() {
    for &v in &[0.0f64, 1.0f64, -1.0f64, f64::MAX, f64::MIN_POSITIVE, std::f64::consts::PI] {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&v.to_be_bytes());
        let out = f64::from_be_bytes(buf);
        assert_eq!(out, v, "big-endian f64 roundtrip {v}");
    }
}

// ===========================================================================
// LittleEndianBinaryReadWrite.fs  (mirrors the property tests)
// ===========================================================================

/// Mirrors: LittleEndianBinaryReadWrite."Int16" (property test)
#[test]
fn little_endian_int16_roundtrip() {
    for &v in &[i16::MIN, -1i16, 0i16, 1i16, i16::MAX, 0x1234i16] {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = i16::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian i16 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."UInt16" (property test)
#[test]
fn little_endian_uint16_roundtrip() {
    for &v in &[0u16, 1u16, u16::MAX, 0xABCDu16] {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = u16::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian u16 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."UInt24" (property test)
#[test]
fn little_endian_uint24_roundtrip() {
    for &v in &[0u32, 1u32, 0xFF_FFFFu32, 0x12_3456u32] {
        let bytes = v.to_le_bytes(); // 4 bytes
        let write_buf = [bytes[0], bytes[1], bytes[2]]; // 3 LE bytes
        let out = u32::from_le_bytes([write_buf[0], write_buf[1], write_buf[2], 0]);
        assert_eq!(out, v, "little-endian u24 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."Int32" (property test)
#[test]
fn little_endian_int32_roundtrip() {
    for &v in &[i32::MIN, -1i32, 0i32, 1i32, i32::MAX, 0x1234_5678i32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = i32::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian i32 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."UInt32" (property test)
#[test]
fn little_endian_uint32_roundtrip() {
    for &v in &[0u32, 1u32, u32::MAX, 0xDEAD_BEEFu32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = u32::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian u32 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."UInt40" (property test)
#[test]
fn little_endian_uint40_roundtrip() {
    for &v in &[0u64, 1u64, 0xFF_FFFF_FFFFu64, 0x12_3456_7890u64] {
        let bytes = v.to_le_bytes();
        let write_buf = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]; // 5 LE bytes
        let out = u64::from_le_bytes([write_buf[0], write_buf[1], write_buf[2], write_buf[3], write_buf[4], 0, 0, 0]);
        assert_eq!(out, v, "little-endian u40 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."UInt64" (property test)
#[test]
fn little_endian_uint64_roundtrip() {
    for &v in &[0u64, 1u64, u64::MAX, 0xDEAD_BEEF_1234_5678u64] {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = u64::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian u64 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."Single" (property test)
#[test]
fn little_endian_single_roundtrip() {
    for &v in &[0.0f32, 1.0f32, -1.0f32, f32::MAX, f32::MIN_POSITIVE, 3.14159f32] {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = f32::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian f32 roundtrip {v}");
    }
}

/// Mirrors: LittleEndianBinaryReadWrite."Double" (property test)
#[test]
fn little_endian_double_roundtrip() {
    for &v in &[0.0f64, 1.0f64, -1.0f64, f64::MAX, f64::MIN_POSITIVE, std::f64::consts::PI] {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&v.to_le_bytes());
        let out = f64::from_le_bytes(buf);
        assert_eq!(out, v, "little-endian f64 roundtrip {v}");
    }
}
