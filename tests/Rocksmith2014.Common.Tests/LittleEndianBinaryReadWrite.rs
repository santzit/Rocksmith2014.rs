//! Little-Endian Binary Writer/Reader round-trip tests.
//!
//! Mirrors `LittleEndianBinaryReadWrite.fs` in Rocksmith2014.Common.Tests (.NET).
//!
//! The F# tests use FsCheck property-based testing (random values).
//! Here we test a representative set of values for each type.

use rocksmith2014_common::binary_readers::*;
use rocksmith2014_common::binary_writers::*;
use std::io::Cursor;

macro_rules! le_round_trip {
    ($name:ident, $write:ident, $read:ident, $vals:expr) => {
        #[test]
        fn $name() {
            for v in $vals {
                let mut buf = Vec::new();
                $write(&mut buf, v).unwrap();
                let mut cur = Cursor::new(&buf);
                let res = $read(&mut cur).unwrap();
                assert_eq!(res, v, "Same value");
            }
        }
    };
}

// testProp "Int16"
le_round_trip!(
    int16,
    write_i16,
    read_i16,
    [0i16, 1, -1, i16::MIN, i16::MAX, 0x1234]
);

// testProp "UInt16"
le_round_trip!(
    uint16,
    write_u16,
    read_u16,
    [0u16, 1, u16::MAX, 0x1234, 0xABCD]
);

// testProp "UInt24"  (3-byte value, fits in u32, max 0xFFFFFF)
le_round_trip!(
    uint24,
    write_u24,
    read_u24,
    [0u32, 1, 0xFFFFFF, 0x123456, 0x800000]
);

// testProp "Int32"
le_round_trip!(
    int32,
    write_i32,
    read_i32,
    [0i32, 1, -1, i32::MIN, i32::MAX, 0x12345678]
);

// testProp "UInt32"
le_round_trip!(
    uint32,
    write_u32,
    read_u32,
    [0u32, 1, u32::MAX, 0x12345678, 0xDEADBEEF]
);

// testProp "UInt40"  (5-byte value, fits in u64, max 0xFFFFFFFFFF)
le_round_trip!(
    uint40,
    write_u40,
    read_u40,
    [0u64, 1, 0xFF_FFFF_FFFF, 0x12_3456_7890]
);

// testProp "UInt64"
le_round_trip!(
    uint64,
    write_u64,
    read_u64,
    [0u64, 1, u64::MAX, 0x0123456789ABCDEF]
);

// testProp "Single"
#[test]
fn single() {
    for v in [0.0f32, 1.0, -1.0, f32::MIN_POSITIVE, f32::MAX, 1.234_567] {
        let mut buf = Vec::new();
        write_f32(&mut buf, v).unwrap();
        let mut cur = Cursor::new(&buf);
        let res = read_f32(&mut cur).unwrap();
        assert_eq!(res, v, "Same value");
    }
}

// testProp "Double"
#[test]
fn double() {
    for v in [
        0.0f64,
        1.0,
        -1.0,
        f64::MIN_POSITIVE,
        f64::MAX,
        1.234_567_890_123,
    ] {
        let mut buf = Vec::new();
        write_f64(&mut buf, v).unwrap();
        let mut cur = Cursor::new(&buf);
        let res = read_f64(&mut cur).unwrap();
        assert_eq!(res, v, "Same value");
    }
}
