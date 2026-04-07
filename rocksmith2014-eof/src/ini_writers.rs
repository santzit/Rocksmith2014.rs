use crate::types::IniString;
use crate::write_utils::*;
use std::io::{self, Write};

pub fn write_ini_strings(writer: &mut impl Write, strings: &[IniString]) -> io::Result<()> {
    write_u16_le(writer, strings.len() as u16)?;
    for s in strings {
        write_u8(writer, s.string_type as u8)?;
        write_eof_string(writer, &s.value)?;
    }
    Ok(())
}

pub fn write_ini_booleans(writer: &mut impl Write) -> io::Result<()> {
    write_u16_le(writer, 1u16)?;
    write_u8(writer, 11u8 | (1u8 << 7))?;
    Ok(())
}

pub fn write_ini_numbers(writer: &mut impl Write) -> io::Result<()> {
    write_u16_le(writer, 1u16)?;
    write_u8(writer, 2u8)?;
    write_u32_le(writer, 255u32)?;
    Ok(())
}
