//! Tests mirroring Rocksmith2014.FSharpExtensions.Tests (ExtensionTests, ActivePatternTests).

use rocksmith2014_fsharp_extensions::{
    array_ext::{all_same, choose_indexed, collect_indexed, try_average},
    file_ext::try_map,
    option_ext::{of_slice, of_str},
    string_ext::{
        contains, contains_ignore_case, ends_with_ignore_case, equals_ignore_case, not_empty,
        starts_with_ignore_case, trim, truncate,
    },
    vec_ext::{
        filter, find_max_by, init, remove, singleton, try_find, try_head, try_item, try_last,
        try_pick,
    },
};

// ---------------------------------------------------------------------------
// String tests
// ---------------------------------------------------------------------------

#[test]
fn not_empty_returns_true_for_non_blank() {
    assert!(not_empty("hello"));
    assert!(!not_empty("   "));
    assert!(!not_empty(""));
}

#[test]
fn equals_ignore_case_is_case_insensitive() {
    assert!(equals_ignore_case("Hello", "hello"));
    assert!(equals_ignore_case("WORLD", "world"));
    assert!(!equals_ignore_case("foo", "bar"));
}

#[test]
fn starts_with_ignore_case_works() {
    assert!(starts_with_ignore_case("hel", "Hello World"));
    assert!(!starts_with_ignore_case("world", "Hello World"));
}

#[test]
fn ends_with_ignore_case_works() {
    assert!(ends_with_ignore_case("WORLD", "Hello World"));
    assert!(!ends_with_ignore_case("hello", "Hello World"));
}

#[test]
fn contains_is_case_sensitive() {
    assert!(contains("World", "Hello World"));
    assert!(!contains("world", "Hello World"));
}

#[test]
fn contains_ignore_case_is_case_insensitive() {
    assert!(contains_ignore_case("world", "Hello World"));
    assert!(contains_ignore_case("WORLD", "Hello World"));
}

#[test]
fn truncate_shortens_long_string() {
    assert_eq!(truncate(3, "hello"), "hel");
    assert_eq!(truncate(10, "hi"), "hi");
}

#[test]
fn trim_removes_whitespace() {
    assert_eq!(trim("  hello  "), "hello");
}

// ---------------------------------------------------------------------------
// Array tests
// ---------------------------------------------------------------------------

#[test]
fn try_average_returns_zero_for_empty() {
    assert_eq!(try_average(&[] as &[f32]), 0.0);
}

#[test]
fn try_average_returns_mean() {
    assert_eq!(try_average(&[1.0f32, 3.0]), 2.0);
}

#[test]
fn all_same_with_identical_elements() {
    assert!(all_same(&[5, 5, 5]));
    assert!(all_same::<i32>(&[]));
}

#[test]
fn all_same_with_different_elements() {
    assert!(!all_same(&[5, 5, 6]));
}

#[test]
fn choose_indexed_passes_correct_index() {
    let arr = [10, 20, 30];
    let result = choose_indexed(&arr, |i, &x| if i < 2 { Some(x + i as i32) } else { None });
    assert_eq!(result, vec![10, 21]);
}

#[test]
fn collect_indexed_flattens_with_index() {
    let result = collect_indexed(&[1i32, 2], |i, &x| vec![i as i32, x]);
    assert_eq!(result, vec![0, 1, 1, 2]);
}

// ---------------------------------------------------------------------------
// Vec (ResizeArray) tests
// ---------------------------------------------------------------------------

#[test]
fn init_builds_vec_with_initializer() {
    let v = init(4, |i| i * 3);
    assert_eq!(v, vec![0, 3, 6, 9]);
}

#[test]
fn try_item_in_bounds() {
    let v = vec![10, 20, 30];
    assert_eq!(try_item(1, &v), Some(&20));
    assert_eq!(try_item(5, &v), None);
}

#[test]
fn try_last_and_try_head() {
    let v = vec![1, 2, 3];
    assert_eq!(try_head(&v), Some(&1));
    assert_eq!(try_last(&v), Some(&3));
    let empty: Vec<i32> = vec![];
    assert_eq!(try_head(&empty), None);
    assert_eq!(try_last(&empty), None);
}

#[test]
fn try_find_returns_first_match() {
    let v = vec![1, 2, 3, 4];
    assert_eq!(try_find(&v, |&x| x > 2), Some(&3));
    assert_eq!(try_find(&v, |&x| x > 10), None);
}

#[test]
fn try_pick_returns_first_some() {
    let v = vec![1, 2, 3];
    let r = try_pick(&v, |&x| if x == 2 { Some(x * 10) } else { None });
    assert_eq!(r, Some(20));
}

#[test]
fn filter_keeps_matching_elements() {
    let v = vec![1, 2, 3, 4, 5];
    let even = filter(&v, |&x| x % 2 == 0);
    assert_eq!(even, vec![2, 4]);
}

#[test]
fn singleton_wraps_value() {
    assert_eq!(singleton(42), vec![42]);
}

#[test]
fn find_max_by_returns_maximum_projection() {
    let v = vec!["a", "bbb", "cc"];
    let max_len = find_max_by(&v, |s| s.len());
    assert_eq!(max_len, 3);
}

#[test]
fn remove_removes_first_occurrence() {
    let mut v = vec!["a", "b", "c", "b"];
    remove(&mut v, &"b");
    assert_eq!(v, vec!["a", "c", "b"]);
}

// ---------------------------------------------------------------------------
// Option tests
// ---------------------------------------------------------------------------

#[test]
fn of_str_returns_none_for_blank() {
    assert_eq!(of_str(""), None);
    assert_eq!(of_str("  "), None);
    assert_eq!(of_str("hello"), Some("hello"));
}

#[test]
fn of_slice_returns_none_for_empty() {
    let empty: &[i32] = &[];
    assert_eq!(of_slice(empty), None);
    assert_eq!(of_slice(&[1, 2][..]), Some(&[1, 2][..]));
}

// ---------------------------------------------------------------------------
// File tests
// ---------------------------------------------------------------------------

#[test]
fn try_map_returns_some_for_existing_file() {
    // Use this test binary's own Cargo.toml which always exists.
    let cargo = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let result = try_map(cargo, |_| "exists");
    assert_eq!(result, Some("exists"));
}

#[test]
fn try_map_returns_none_for_missing_file() {
    let result = try_map("/no/such/file.xyz", |_| "found");
    assert_eq!(result, None);
}
