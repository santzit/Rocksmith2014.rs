//! String utilities.
//!
//! Ports `Rocksmith2014.FSharpExtensions.String`.

/// Returns `true` if the string is non-empty (not blank / whitespace-only).
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::string_ext::not_empty;
/// assert!(not_empty("hello"));
/// assert!(!not_empty("   "));
/// assert!(!not_empty(""));
/// ```
pub fn not_empty(s: &str) -> bool {
    !s.trim().is_empty()
}

/// Returns `true` if both strings are equal ignoring ASCII case.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Returns `true` if `s` starts with `prefix` (case-insensitive).
pub fn starts_with_ignore_case(prefix: &str, s: &str) -> bool {
    s.to_ascii_lowercase()
        .starts_with(&prefix.to_ascii_lowercase())
}

/// Returns `true` if `s` ends with `suffix` (case-insensitive).
pub fn ends_with_ignore_case(suffix: &str, s: &str) -> bool {
    s.to_ascii_lowercase()
        .ends_with(&suffix.to_ascii_lowercase())
}

/// Returns `true` if `s` contains `substr` (case-sensitive).
pub fn contains(substr: &str, s: &str) -> bool {
    s.contains(substr)
}

/// Returns `true` if `s` contains `substr` (case-insensitive).
pub fn contains_ignore_case(substr: &str, s: &str) -> bool {
    s.to_ascii_lowercase()
        .contains(&substr.to_ascii_lowercase())
}

/// Returns the string truncated to `max_len` characters (by char boundary).
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::string_ext::truncate;
/// assert_eq!(truncate(3, "hello"), "hel");
/// assert_eq!(truncate(10, "hi"), "hi");
/// ```
pub fn truncate(max_len: usize, s: &str) -> &str {
    if s.len() > max_len {
        &s[..max_len]
    } else {
        s
    }
}

/// Trims leading and trailing whitespace.
pub fn trim(s: &str) -> &str {
    s.trim()
}
