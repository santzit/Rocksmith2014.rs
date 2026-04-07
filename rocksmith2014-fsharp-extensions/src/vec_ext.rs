//! `Vec` utilities — mirrors the F# `ResizeArray` module.
//!
//! Ports `Rocksmith2014.FSharpExtensions.ResizeArray`.

/// Creates a `Vec<U>` of length `size` by calling `init(i)` for each index.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::vec_ext::init;
/// let v = init(3, |i| i * 2);
/// assert_eq!(v, vec![0, 2, 4]);
/// ```
pub fn init<U, F: FnMut(usize) -> U>(size: usize, f: F) -> Vec<U> {
    (0..size).map(f).collect()
}

/// Returns `Some(&v[index])` if `index` is in bounds, otherwise `None`.
pub fn try_item<T>(index: usize, v: &[T]) -> Option<&T> {
    v.get(index)
}

/// Returns `Some` of the last element, or `None` for an empty slice.
pub fn try_last<T>(v: &[T]) -> Option<&T> {
    v.last()
}

/// Returns `Some` of the first element, or `None` for an empty slice.
pub fn try_head<T>(v: &[T]) -> Option<&T> {
    v.first()
}

/// Returns the first element satisfying `predicate`, or `None`.
pub fn try_find<T, F: FnMut(&T) -> bool>(v: &[T], mut predicate: F) -> Option<&T> {
    v.iter().find(|x| predicate(x))
}

/// Returns the first `Some` value from `chooser`, or `None`.
pub fn try_pick<T, U, F: FnMut(&T) -> Option<U>>(v: &[T], chooser: F) -> Option<U> {
    v.iter().find_map(chooser)
}

/// Returns a `Vec` containing only elements where `predicate` is `true`.
pub fn filter<T: Clone, F: FnMut(&T) -> bool>(v: &[T], mut predicate: F) -> Vec<T> {
    v.iter().filter(|x| predicate(x)).cloned().collect()
}

/// Returns a `Vec` containing a single element.
pub fn singleton<T>(value: T) -> Vec<T> {
    vec![value]
}

/// Returns the largest value found using `projection`.
///
/// # Panics
/// Panics on an empty slice.
pub fn find_max_by<T, U: PartialOrd, F: FnMut(&T) -> U>(v: &[T], projection: F) -> U {
    v.iter()
        .map(projection)
        .reduce(|a, b| if b > a { b } else { a })
        .expect("find_max_by called on empty slice")
}

/// Removes the first occurrence of `item` from a `Vec` (by equality).
pub fn remove<T: PartialEq>(v: &mut Vec<T>, item: &T) {
    if let Some(pos) = v.iter().position(|x| x == item) {
        v.remove(pos);
    }
}
