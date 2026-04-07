//! Array / slice utilities.
//!
//! Ports `Rocksmith2014.FSharpExtensions.Array`.

/// Returns the average of the slice, or `0.0` for an empty slice.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::array_ext::try_average;
/// assert_eq!(try_average(&[1.0f32, 3.0]), 2.0);
/// assert_eq!(try_average(&[] as &[f32]), 0.0);
/// ```
pub fn try_average(arr: &[f32]) -> f32 {
    if arr.is_empty() {
        0.0
    } else {
        arr.iter().sum::<f32>() / arr.len() as f32
    }
}

/// Returns `true` if every element in the slice is equal, or the slice is empty.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::array_ext::all_same;
/// assert!(all_same(&[5, 5, 5]));
/// assert!(!all_same(&[5, 5, 6]));
/// assert!(all_same::<i32>(&[]));
/// ```
pub fn all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

/// Like `Iterator::filter_map` but the closure also receives the element index.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::array_ext::choose_indexed;
/// let v = choose_indexed(&[10, 20, 30], |i, &x| if i < 2 { Some(x) } else { None });
/// assert_eq!(v, vec![10, 20]);
/// ```
pub fn choose_indexed<T, U, F>(arr: &[T], mut f: F) -> Vec<U>
where
    F: FnMut(usize, &T) -> Option<U>,
{
    arr.iter()
        .enumerate()
        .filter_map(|(i, x)| f(i, x))
        .collect()
}

/// Like `Iterator::flat_map` but the closure also receives the element index.
///
/// # Example
/// ```
/// # use rocksmith2014_fsharp_extensions::array_ext::collect_indexed;
/// let v = collect_indexed(&[1, 2], |i, &x| vec![i as i32, x]);
/// assert_eq!(v, vec![0, 1, 1, 2]);
/// ```
pub fn collect_indexed<T, U, F>(arr: &[T], mut f: F) -> Vec<U>
where
    F: FnMut(usize, &T) -> Vec<U>,
{
    arr.iter().enumerate().flat_map(|(i, x)| f(i, x)).collect()
}
