//! Tests for the [`HandShape`] type.
//!
//! Mirrors `HandShapeTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::HandShape;

/// Mirrors `HandShapeTests.CopyConstructorCopiesAllValues`
#[test]
fn copy_constructor_copies_all_values() {
    let hs1 = HandShape {
        chord_id: 15,
        start_time: 7777,
        end_time: 8888,
    };

    let hs2 = hs1.clone();

    assert_eq!(hs2.chord_id, 15);
    assert_eq!(hs2.start_time, 7777);
    assert_eq!(hs2.end_time, 8888);
}
