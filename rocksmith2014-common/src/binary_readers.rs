//! Binary reading utilities for little-endian and big-endian primitive types.

use std::io::{self, Read};

pub fn read_u8<R: Read>(r: &mut R) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_i8<R: Read>(r: &mut R) -> io::Result<i8> {
    Ok(read_u8(r)? as i8)
}

// ---------------------------------------------------------------------------
// Little-endian
// ---------------------------------------------------------------------------

pub fn read_i16<R: Read>(r: &mut R) -> io::Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

pub fn read_u16<R: Read>(r: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Reads 3 bytes in little-endian order and returns a `u32`.
pub fn read_u24<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 3];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], 0]))
}

pub fn read_i32<R: Read>(r: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

pub fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Reads 5 bytes in little-endian order and returns a `u64`.
pub fn read_u40<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 5];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], 0, 0, 0,
    ]))
}

pub fn read_u64<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn read_f32<R: Read>(r: &mut R) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub fn read_f64<R: Read>(r: &mut R) -> io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

/// Reads exactly `N` bytes into a fixed-size array.
pub fn read_fixed_string<R: Read, const N: usize>(r: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Big-endian
// ---------------------------------------------------------------------------

pub fn read_i16_be<R: Read>(r: &mut R) -> io::Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

pub fn read_u16_be<R: Read>(r: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

/// Reads 3 bytes in big-endian order and returns a `u32`.
pub fn read_u24_be<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 3];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
}

pub fn read_i32_be<R: Read>(r: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

pub fn read_u32_be<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Reads 5 bytes in big-endian order and returns a `u64`.
pub fn read_u40_be<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 5];
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes([
        0, 0, 0, buf[0], buf[1], buf[2], buf[3], buf[4],
    ]))
}

pub fn read_u64_be<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub fn read_f32_be<R: Read>(r: &mut R) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

pub fn read_f64_be<R: Read>(r: &mut R) -> io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}
