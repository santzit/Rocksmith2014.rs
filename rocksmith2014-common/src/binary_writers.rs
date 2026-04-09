//! Binary writing utilities for little-endian and big-endian primitive types.

use std::io::{self, Write};

pub fn write_u8<W: Write>(w: &mut W, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

pub fn write_i8<W: Write>(w: &mut W, v: i8) -> io::Result<()> {
    w.write_all(&[v as u8])
}

// ---------------------------------------------------------------------------
// Little-endian
// ---------------------------------------------------------------------------

pub fn write_i16<W: Write>(w: &mut W, v: i16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u16<W: Write>(w: &mut W, v: u16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

/// Writes the low 24 bits of `v` in little-endian order (3 bytes).
pub fn write_u24<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    let b = v.to_le_bytes();
    w.write_all(&b[..3])
}

pub fn write_i32<W: Write>(w: &mut W, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u32<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

/// Writes the low 40 bits of `v` in little-endian order (5 bytes).
pub fn write_u40<W: Write>(w: &mut W, v: u64) -> io::Result<()> {
    let b = v.to_le_bytes();
    w.write_all(&b[..5])
}

pub fn write_u64<W: Write>(w: &mut W, v: u64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_f32<W: Write>(w: &mut W, v: f32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_f64<W: Write>(w: &mut W, v: f64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

/// Writes a fixed-size byte array (fixed-length string field).
pub fn write_fixed_string<W: Write, const N: usize>(w: &mut W, s: &[u8; N]) -> io::Result<()> {
    w.write_all(s)
}

// ---------------------------------------------------------------------------
// Big-endian
// ---------------------------------------------------------------------------

pub fn write_i16_be<W: Write>(w: &mut W, v: i16) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

pub fn write_u16_be<W: Write>(w: &mut W, v: u16) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

/// Writes the low 24 bits of `v` in big-endian order (3 bytes).
pub fn write_u24_be<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    let b = v.to_be_bytes();
    w.write_all(&b[1..])
}

pub fn write_i32_be<W: Write>(w: &mut W, v: i32) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

pub fn write_u32_be<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

/// Writes the low 40 bits of `v` in big-endian order (5 bytes).
pub fn write_u40_be<W: Write>(w: &mut W, v: u64) -> io::Result<()> {
    let b = v.to_be_bytes();
    w.write_all(&b[3..])
}

pub fn write_u64_be<W: Write>(w: &mut W, v: u64) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

pub fn write_f32_be<W: Write>(w: &mut W, v: f32) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

pub fn write_f64_be<W: Write>(w: &mut W, v: f64) -> io::Result<()> {
    w.write_all(&v.to_be_bytes())
}
