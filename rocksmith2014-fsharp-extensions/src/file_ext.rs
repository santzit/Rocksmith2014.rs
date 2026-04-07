//! File utilities.
//!
//! Ports `Rocksmith2014.FSharpExtensions.Misc` (File section).

use std::path::Path;

/// Calls `f(path)` if the file exists and returns `Some(result)`, otherwise `None`.
///
/// # Example
/// ```no_run
/// # use rocksmith2014_fsharp_extensions::file_ext::try_map;
/// let len = try_map("Cargo.toml", |p| std::fs::metadata(p).unwrap().len());
/// assert!(len.is_some());
/// ```
pub fn try_map<T, F: FnOnce(&str) -> T>(path: &str, f: F) -> Option<T> {
    if Path::new(path).exists() {
        Some(f(path))
    } else {
        None
    }
}
