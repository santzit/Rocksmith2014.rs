//! Tests for the [`Anchor`] type.
//!
//! Mirrors `AnchorTests.cs` in Rocksmith2014.NET.

use rocksmith2014_xml::Anchor;

/// Mirrors `AnchorTests.CopyConstructorCopiesAllValues`
#[test]
fn copy_constructor_copies_all_values() {
    let a1 = Anchor {
        fret: 22,
        time: 4567,
        width: 6,
        end_time: 0,
    };
    let a2 = a1.clone();

    assert_eq!(a2.fret, 22);
    assert_eq!(a2.time, 4567);
    assert_eq!(a2.width, 6);
    assert_eq!(a1, a2);
}

/// Mirrors `AnchorTests.UsesStructuralEquality`
#[test]
fn uses_structural_equality() {
    let a1 = Anchor {
        fret: 22,
        time: 4567,
        width: 6,
        end_time: 0,
    };
    let a2 = Anchor {
        fret: 22,
        time: 4567,
        width: 6,
        end_time: 0,
    };

    assert_eq!(a1, a2);
    assert_ne!(a1, Anchor { fret: 1, ..a2 });
}
