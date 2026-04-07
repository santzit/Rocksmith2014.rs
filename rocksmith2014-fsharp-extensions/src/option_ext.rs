//! Option utilities.
//!
//! Ports `Rocksmith2014.FSharpExtensions.Misc` (Option section).

/// Creates an `Option<&str>` from a string slice: `None` for blank/empty.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::option_ext::of_str;
/// assert_eq!(of_str(""), None);
/// assert_eq!(of_str("   "), None);
/// assert_eq!(of_str("hello"), Some("hello"));
/// ```
pub fn of_str(s: &str) -> Option<&str> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Creates an `Option<&[T]>` from a slice: `None` for an empty slice.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::option_ext::of_slice;
/// let empty: &[i32] = &[];
/// assert_eq!(of_slice(empty), None);
/// assert_eq!(of_slice(&[1, 2]), Some(&[1, 2][..]));
/// ```
pub fn of_slice<T>(arr: &[T]) -> Option<&[T]> {
    if arr.is_empty() {
        None
    } else {
        Some(arr)
    }
}
