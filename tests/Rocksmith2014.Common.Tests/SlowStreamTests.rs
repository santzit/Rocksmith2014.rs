//! Binary reader tests with a stream that returns one byte at a time.
//!
//! Mirrors `SlowStreamTests.fs` in Rocksmith2014.Common.Tests (.NET).
//!
//! The slow stream verifies that the readers loop correctly when the
//! underlying `Read::read` returns less data than requested.

use rocksmith2014_common::binary_readers::*;
use std::io::{self, Read};

/// A `Read` implementation that returns exactly one byte per `read` call.
/// Data is the sequence of bytes 0, 1, 2, … (index as byte, wraps at 256).
struct SlowStream {
    index: usize,
}

impl SlowStream {
    fn new() -> Self {
        SlowStream { index: 0 }
    }
}

impl Read for SlowStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        buf[0] = self.index as u8;
        self.index += 1;
        Ok(1)
    }
}

// ---------------------------------------------------------------------------
// Big-Endian Binary Reader with Slow Stream
// ---------------------------------------------------------------------------

/// test "Can read bytes"
///
/// Expect.sequenceContainsOrder array (seq { 0uy; 1uy; 2uy; 3uy; 4uy }) "..."
#[test]
fn be_can_read_bytes() {
    let mut stream = SlowStream::new();
    let mut bytes = [0u8; 5];
    stream.read_exact(&mut bytes).unwrap();
    assert_eq!(bytes, [0, 1, 2, 3, 4], "Sequence of 5 bytes is correct");
}

/// test "Can read signed 16-bit integer"
///
/// Bytes from stream: [0, 1]
/// Big-endian Int16 = 0x0001 = 1
/// expected = BitConverter.ToInt16([| 1uy; 0uy |], 0) = 1  (little-endian interpretation of [1,0])
#[test]
fn be_can_read_int16() {
    let mut stream = SlowStream::new();
    let res = read_i16_be(&mut stream).unwrap();
    let expected = i16::from_le_bytes([1, 0]);
    assert_eq!(res, expected, "Signed 16-bit integer read correctly");
}

/// test "Can read unsigned 24-bit integer"
///
/// Bytes from stream: [0, 1, 2]
/// Big-endian UInt24 = 0x000102 = 258
/// expected = BitConverter.ToUInt32([| 2uy; 1uy; 0uy; 0uy |], 0) = 258 (little-endian)
#[test]
fn be_can_read_uint24() {
    let mut stream = SlowStream::new();
    let res = read_u24_be(&mut stream).unwrap();
    let expected = u32::from_le_bytes([2, 1, 0, 0]);
    assert_eq!(res, expected, "Unsigned 24-bit integer read correctly");
}

/// test "Can read unsigned 64-bit integer"
///
/// Bytes from stream: [0, 1, 2, 3, 4, 5, 6, 7]
/// Big-endian UInt64 = 0x0001020304050607
/// expected = BitConverter.ToUInt64([| 7uy; 6uy; 5uy; 4uy; 3uy; 2uy; 1uy; 0uy |], 0)
///          = 0x0001020304050607 (little-endian of reversed bytes)
#[test]
fn be_can_read_uint64() {
    let mut stream = SlowStream::new();
    let res = read_u64_be(&mut stream).unwrap();
    let expected = u64::from_le_bytes([7, 6, 5, 4, 3, 2, 1, 0]);
    assert_eq!(res, expected, "Unsigned 64-bit integer read correctly");
}

// ---------------------------------------------------------------------------
// Little-Endian Binary Reader with Slow Stream
// ---------------------------------------------------------------------------

/// test "Can read bytes"
#[test]
fn le_can_read_bytes() {
    let mut stream = SlowStream::new();
    let mut bytes = [0u8; 5];
    stream.read_exact(&mut bytes).unwrap();
    assert_eq!(bytes, [0, 1, 2, 3, 4], "Sequence of 5 bytes is correct");
}

/// test "Can read signed 16-bit integer"
///
/// Bytes from stream: [0, 1]
/// Little-endian Int16 = 0x0100 = 256
/// expected = BitConverter.ToInt16([| 0uy; 1uy |], 0) = 256
#[test]
fn le_can_read_int16() {
    let mut stream = SlowStream::new();
    let res = read_i16(&mut stream).unwrap();
    let expected = i16::from_le_bytes([0, 1]);
    assert_eq!(res, expected, "Signed 16-bit integer read correctly");
}

/// test "Can read unsigned 24-bit integer"
///
/// Bytes from stream: [0, 1, 2]
/// Little-endian UInt24 = 0x020100 = 131328
/// expected = BitConverter.ToUInt32([| 0uy; 1uy; 2uy; 0uy |], 0) = 131328
#[test]
fn le_can_read_uint24() {
    let mut stream = SlowStream::new();
    let res = read_u24(&mut stream).unwrap();
    let expected = u32::from_le_bytes([0, 1, 2, 0]);
    assert_eq!(res, expected, "Unsigned 24-bit integer read correctly");
}

/// test "Can read unsigned 64-bit integer"
///
/// Bytes from stream: [0, 1, 2, 3, 4, 5, 6, 7]
/// Little-endian UInt64 = 0x0706050403020100
/// expected = BitConverter.ToUInt64([| 0uy; 1uy; 2uy; 3uy; 4uy; 5uy; 6uy; 7uy |], 0)
///          = 0x0706050403020100
#[test]
fn le_can_read_uint64() {
    let mut stream = SlowStream::new();
    let res = read_u64(&mut stream).unwrap();
    let expected = u64::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(res, expected, "Unsigned 64-bit integer read correctly");
}
