//! C-compatible FFI layer for PSARC, SNG and XML.
//!
//! All functions follow these conventions:
//! - Return `null` on error (pointer-returning functions).
//! - Return `0` on success, `-1` on error (int-returning functions).
//! - Callers must use the matching `_close` / `_free_*` function to release
//!   every allocation made by this library.

use std::{
    ffi::{CStr},
    os::raw::{c_char, c_int},
    path::PathBuf,
    slice,
};

use crate::{
    psarc::Psarc,
    sng::{Platform, types::Sng},
    xml::InstrumentalArrangement,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a `*const c_char` to a `PathBuf`, returning `None` if null/invalid.
unsafe fn path_from_ptr(ptr: *const c_char) -> Option<PathBuf> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr)
        .to_str()
        .ok()
        .map(PathBuf::from)
}

// ===========================================================================
// PSARC
// ===========================================================================

/// Open a PSARC archive from `path`.
///
/// Returns an opaque handle, or `null` on failure.
/// The caller must call [`rs2014_psarc_close`] when done.
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_open(path: *const c_char) -> *mut Psarc {
    let Some(p) = path_from_ptr(path) else { return std::ptr::null_mut() };
    match Psarc::open(p) {
        Ok(psarc) => Box::into_raw(Box::new(psarc)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Close a PSARC handle previously opened with [`rs2014_psarc_open`].
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_close(psarc: *mut Psarc) {
    if !psarc.is_null() {
        drop(Box::from_raw(psarc));
    }
}

/// Return the number of entries in the archive.
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_entry_count(psarc: *const Psarc) -> c_int {
    if psarc.is_null() {
        return -1;
    }
    (*psarc).entry_count() as c_int
}

/// Return the null-terminated name of entry at `index`.
///
/// The returned pointer is valid for the lifetime of the `Psarc` handle.
/// Returns `null` if `index` is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_entry_name(
    psarc: *const Psarc,
    index: c_int,
) -> *const c_char {
    if psarc.is_null() || index < 0 {
        return std::ptr::null();
    }
    (*psarc).name_ptr(index as usize)
}

/// Extract the entry at `index` into a newly allocated buffer.
///
/// On success writes the buffer pointer to `*out_data` and the byte length to
/// `*out_len`, then returns `0`.  Returns `-1` on failure.
/// The buffer must be freed with [`rs2014_psarc_free_data`].
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_extract(
    psarc: *const Psarc,
    index: c_int,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if psarc.is_null() || index < 0 || out_data.is_null() || out_len.is_null() {
        return -1;
    }
    match (*psarc).extract(index as usize) {
        Ok(bytes) => {
            *out_len = bytes.len();
            let mut v = bytes.into_boxed_slice();
            *out_data = v.as_mut_ptr();
            std::mem::forget(v);
            0
        }
        Err(_) => -1,
    }
}

/// Free a buffer previously returned by [`rs2014_psarc_extract`].
#[no_mangle]
pub unsafe extern "C" fn rs2014_psarc_free_data(data: *mut u8, len: usize) {
    if !data.is_null() {
        drop(Box::from_raw(slice::from_raw_parts_mut(data, len)));
    }
}

// ===========================================================================
// SNG
// ===========================================================================

/// Open a packed (encrypted + compressed) SNG file.
///
/// `platform`: `0` = PC, `1` = Mac.
/// Returns an opaque handle, or `null` on failure.
/// The caller must call [`rs2014_sng_close`] when done.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_open_packed(
    path: *const c_char,
    platform: c_int,
) -> *mut Sng {
    let Some(p) = path_from_ptr(path) else { return std::ptr::null_mut() };
    let plat = Platform::from_i32(platform);
    match Sng::read_packed_file(p, plat) {
        Ok(sng) => Box::into_raw(Box::new(sng)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Open an unpacked (plain binary) SNG file.
///
/// Returns an opaque handle, or `null` on failure.
/// The caller must call [`rs2014_sng_close`] when done.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_open_unpacked(path: *const c_char) -> *mut Sng {
    let Some(p) = path_from_ptr(path) else { return std::ptr::null_mut() };
    match Sng::read_unpacked_file(p) {
        Ok(sng) => Box::into_raw(Box::new(sng)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Close an SNG handle.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_close(sng: *mut Sng) {
    if !sng.is_null() {
        drop(Box::from_raw(sng));
    }
}

/// Return the number of beats in the SNG.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_beat_count(sng: *const Sng) -> c_int {
    if sng.is_null() { return -1; }
    (*sng).beats.len() as c_int
}

/// Return the number of difficulty levels in the SNG.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_level_count(sng: *const Sng) -> c_int {
    if sng.is_null() { return -1; }
    (*sng).levels.len() as c_int
}

/// Return the total song length in seconds.
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_song_length(sng: *const Sng) -> f32 {
    if sng.is_null() { return 0.0; }
    (*sng).metadata.song_length
}

/// Return the number of notes in the hardest difficulty level (or 0).
#[no_mangle]
pub unsafe extern "C" fn rs2014_sng_note_count(sng: *const Sng) -> c_int {
    if sng.is_null() { return -1; }
    (*sng).levels.last().map(|l| l.notes.len() as c_int).unwrap_or(0)
}

// ===========================================================================
// XML
// ===========================================================================

/// Open a Rocksmith 2014 arrangement XML file.
///
/// Returns an opaque handle, or `null` on failure.
/// The caller must call [`rs2014_xml_close`] when done.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_open(path: *const c_char) -> *mut InstrumentalArrangement {
    let Some(p) = path_from_ptr(path) else { return std::ptr::null_mut() };
    match InstrumentalArrangement::open(p) {
        Ok(arr) => Box::into_raw(Box::new(arr)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Close an XML arrangement handle.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_close(arr: *mut InstrumentalArrangement) {
    if !arr.is_null() {
        drop(Box::from_raw(arr));
    }
}

/// Return the song title.  Pointer is valid for the lifetime of the handle.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_title(
    arr: *const InstrumentalArrangement,
) -> *const c_char {
    if arr.is_null() { return std::ptr::null(); }
    (*arr).title_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

/// Return the arrangement name (e.g. "Lead").
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_arrangement(
    arr: *const InstrumentalArrangement,
) -> *const c_char {
    if arr.is_null() { return std::ptr::null(); }
    (*arr).arrangement_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

/// Return the artist name.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_artist_name(
    arr: *const InstrumentalArrangement,
) -> *const c_char {
    if arr.is_null() { return std::ptr::null(); }
    (*arr).artist_name_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

/// Return the artist name sort field.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_artist_name_sort(
    arr: *const InstrumentalArrangement,
) -> *const c_char {
    if arr.is_null() { return std::ptr::null(); }
    (*arr).artist_name_sort_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

/// Return the last conversion date-time string.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_last_conversion_datetime(
    arr: *const InstrumentalArrangement,
) -> *const c_char {
    if arr.is_null() { return std::ptr::null(); }
    (*arr).last_conversion_date_time_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null())
}

/// Return the average tempo in BPM.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_average_tempo(
    arr: *const InstrumentalArrangement,
) -> f32 {
    if arr.is_null() { return 0.0; }
    (*arr).average_tempo
}

/// Return the song length in seconds.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_song_length(
    arr: *const InstrumentalArrangement,
) -> f32 {
    if arr.is_null() { return 0.0; }
    (*arr).song_length
}

/// Return the number of difficulty levels.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_level_count(
    arr: *const InstrumentalArrangement,
) -> c_int {
    if arr.is_null() { return -1; }
    (*arr).level_count
}

/// Return the total note+chord count across all levels.
#[no_mangle]
pub unsafe extern "C" fn rs2014_xml_note_count(
    arr: *const InstrumentalArrangement,
) -> c_int {
    if arr.is_null() { return -1; }
    (*arr).note_count
}
