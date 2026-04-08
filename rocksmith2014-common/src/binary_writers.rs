//! Binary writing utilities for little-endian primitive types.

use std::io::{self, Write};

pub fn write_u8<W: Write>(w: &mut W, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

pub fn write_i8<W: Write>(w: &mut W, v: i8) -> io::Result<()> {
    w.write_all(&[v as u8])
}

pub fn write_i16<W: Write>(w: &mut W, v: i16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u16<W: Write>(w: &mut W, v: u16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_i32<W: Write>(w: &mut W, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn write_u32<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
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
