//! Primitive read/write helpers for the SNG binary format.

use crate::{Error, Result};
use std::io::{self, Read, Write};

pub(crate) fn read_u8<R: Read>(r: &mut R) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub(crate) fn read_i8<R: Read>(r: &mut R) -> io::Result<i8> {
    Ok(read_u8(r)? as i8)
}

pub(crate) fn read_i16<R: Read>(r: &mut R) -> io::Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

pub(crate) fn read_i32<R: Read>(r: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

pub(crate) fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub(crate) fn read_f32<R: Read>(r: &mut R) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub(crate) fn read_f64<R: Read>(r: &mut R) -> io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

pub(crate) fn write_u8<W: Write>(w: &mut W, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

pub(crate) fn write_i8<W: Write>(w: &mut W, v: i8) -> io::Result<()> {
    w.write_all(&[v as u8])
}

pub(crate) fn write_i16<W: Write>(w: &mut W, v: i16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub(crate) fn write_i32<W: Write>(w: &mut W, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub(crate) fn write_u32<W: Write>(w: &mut W, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub(crate) fn write_f32<W: Write>(w: &mut W, v: f32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub(crate) fn write_f64<W: Write>(w: &mut W, v: f64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub(crate) fn read_vec_f32<R: Read>(r: &mut R) -> Result<Vec<f32>> {
    let count = read_i32(r)?;
    if count < 0 {
        return Err(Error::InvalidArrayCount(count));
    }
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_f32(r)?);
    }
    Ok(v)
}

pub(crate) fn write_vec_f32<W: Write>(w: &mut W, v: &[f32]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_f32(w, x)?;
    }
    Ok(())
}

pub(crate) fn read_vec_i32<R: Read>(r: &mut R) -> Result<Vec<i32>> {
    let count = read_i32(r)?;
    if count < 0 {
        return Err(Error::InvalidArrayCount(count));
    }
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_i32(r)?);
    }
    Ok(v)
}

pub(crate) fn write_vec_i32<W: Write>(w: &mut W, v: &[i32]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_i32(w, x)?;
    }
    Ok(())
}

pub(crate) fn read_vec_i16<R: Read>(r: &mut R) -> Result<Vec<i16>> {
    let count = read_i32(r)?;
    if count < 0 {
        return Err(Error::InvalidArrayCount(count));
    }
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_i16(r)?);
    }
    Ok(v)
}

pub(crate) fn write_vec_i16<W: Write>(w: &mut W, v: &[i16]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for &x in v {
        write_i16(w, x)?;
    }
    Ok(())
}

pub(crate) trait SngRead: Sized {
    fn sng_read<R: Read>(r: &mut R) -> Result<Self>;
}

pub(crate) trait SngWrite {
    fn sng_write<W: Write>(&self, w: &mut W) -> Result<()>;
}

pub(crate) fn read_array<T: SngRead, R: Read>(r: &mut R) -> Result<Vec<T>> {
    let count = read_i32(r)?;
    if count < 0 {
        return Err(Error::InvalidArrayCount(count));
    }
    let mut v = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(T::sng_read(r)?);
    }
    Ok(v)
}

pub(crate) fn write_array<T: SngWrite, W: Write>(w: &mut W, v: &[T]) -> Result<()> {
    write_i32(w, v.len() as i32)?;
    for x in v {
        x.sng_write(w)?;
    }
    Ok(())
}
