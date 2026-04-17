//! C ABI exports for .NET P/Invoke adapter interop.
//!
//! The public surface intentionally mirrors the Rust implementation without
//! mapping to .NET naming conventions.  When the .NET test suite runs against
//! these exports via the C# adapter, **failing tests pinpoint mismatches**
//! between the Rust and .NET implementations.
//!
//! # Safety
//! All functions that receive raw pointer handles assume the pointer is valid
//! and was produced by the matching constructor exported here.  Null pointers
//! are checked at the entry of every function.

use rocksmith2014_psarc::Psarc;
use rocksmith2014_xml::read_file;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

// ─── XML — InstrumentalArrangement ───────────────────────────────────────────

pub struct ArrangementHandle(rocksmith2014_xml::InstrumentalArrangement);

/// Parse an arrangement from a file path.  Returns null on error.
#[no_mangle]
pub extern "C" fn rs_arrangement_load(path: *const c_char) -> *mut ArrangementHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    match read_file(Path::new(path.as_ref())) {
        Ok(arr) => Box::into_raw(Box::new(ArrangementHandle(arr))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Return the number of difficulty levels. Returns -1 on null handle.
#[no_mangle]
pub extern "C" fn rs_arrangement_level_count(h: *const ArrangementHandle) -> i32 {
    if h.is_null() {
        return -1;
    }
    unsafe { &*h }.0.levels.len() as i32
}

/// Remove dynamic difficulty, keeping only the highest level.
#[no_mangle]
pub extern "C" fn rs_arrangement_remove_dd(h: *mut ArrangementHandle) {
    if h.is_null() {
        return;
    }
    unsafe { &mut *h }.0.remove_dd();
}

/// Free an arrangement handle produced by `rs_arrangement_load`.
#[no_mangle]
pub extern "C" fn rs_arrangement_free(h: *mut ArrangementHandle) {
    if !h.is_null() {
        unsafe { drop(Box::from_raw(h)) };
    }
}

// ─── XML — MetaData ──────────────────────────────────────────────────────────

pub struct MetaDataHandle {
    title: CString,
    average_tempo: f32,
    artist_name_sort: CString,
    last_conversion_datetime: CString,
}

/// Parse metadata from an arrangement file.  Returns null on error.
#[no_mangle]
pub extern "C" fn rs_metadata_read(path: *const c_char) -> *mut MetaDataHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    match read_file(Path::new(path.as_ref())) {
        Ok(arr) => {
            let h = MetaDataHandle {
                title: CString::new(arr.meta.song_name).unwrap_or_default(),
                average_tempo: arr.meta.average_tempo as f32,
                artist_name_sort: CString::new(arr.meta.artist_name_sort).unwrap_or_default(),
                last_conversion_datetime: CString::new(arr.meta.last_conversion_date_time)
                    .unwrap_or_default(),
            };
            Box::into_raw(Box::new(h))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Borrow the title string (owned by the handle, do NOT free separately).
#[no_mangle]
pub extern "C" fn rs_metadata_title(h: *const MetaDataHandle) -> *const c_char {
    if h.is_null() {
        return std::ptr::null();
    }
    unsafe { &*h }.title.as_ptr()
}

/// Average tempo in beats per minute.
#[no_mangle]
pub extern "C" fn rs_metadata_average_tempo(h: *const MetaDataHandle) -> f32 {
    if h.is_null() {
        return 0.0;
    }
    unsafe { &*h }.average_tempo
}

/// Borrow the artist-name-sort string (owned by the handle).
#[no_mangle]
pub extern "C" fn rs_metadata_artist_name_sort(h: *const MetaDataHandle) -> *const c_char {
    if h.is_null() {
        return std::ptr::null();
    }
    unsafe { &*h }.artist_name_sort.as_ptr()
}

/// Borrow the last-conversion-date-time string (owned by the handle).
#[no_mangle]
pub extern "C" fn rs_metadata_last_conversion_datetime(
    h: *const MetaDataHandle,
) -> *const c_char {
    if h.is_null() {
        return std::ptr::null();
    }
    unsafe { &*h }.last_conversion_datetime.as_ptr()
}

/// Free a metadata handle produced by `rs_metadata_read`.
#[no_mangle]
pub extern "C" fn rs_metadata_free(h: *mut MetaDataHandle) {
    if !h.is_null() {
        unsafe { drop(Box::from_raw(h)) };
    }
}

// ─── PSARC ────────────────────────────────────────────────────────────────────

pub struct PsarcHandle {
    psarc: Psarc<std::fs::File>,
}

/// Open a PSARC file. Returns null on error.
#[no_mangle]
pub extern "C" fn rs_psarc_open_file(path: *const c_char) -> *mut PsarcHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    match Psarc::open(Path::new(path.as_ref())) {
        Ok(psarc) => Box::into_raw(Box::new(PsarcHandle { psarc })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Number of entries in the manifest.
#[no_mangle]
pub extern "C" fn rs_psarc_manifest_count(h: *const PsarcHandle) -> i32 {
    if h.is_null() {
        return -1;
    }
    unsafe { &*h }.psarc.manifest().len() as i32
}

/// Return a freshly-allocated C string for manifest entry `idx`.
/// Caller must free with `rs_free_string`.
#[no_mangle]
pub extern "C" fn rs_psarc_manifest_get(h: *const PsarcHandle, idx: i32) -> *mut c_char {
    if h.is_null() || idx < 0 {
        return std::ptr::null_mut();
    }
    let psarc = unsafe { &*h };
    let manifest = psarc.psarc.manifest();
    let idx = idx as usize;
    if idx >= manifest.len() {
        return std::ptr::null_mut();
    }
    CString::new(manifest[idx].as_str())
        .unwrap_or_default()
        .into_raw()
}

/// Number of entries in the table of contents.
#[no_mangle]
pub extern "C" fn rs_psarc_toc_count(h: *const PsarcHandle) -> i32 {
    if h.is_null() {
        return -1;
    }
    unsafe { &*h }.psarc.toc().len() as i32
}

/// Uncompressed length (bytes) of TOC entry `idx`.
#[no_mangle]
pub extern "C" fn rs_psarc_toc_entry_length(h: *const PsarcHandle, idx: i32) -> u64 {
    if h.is_null() || idx < 0 {
        return 0;
    }
    let psarc = unsafe { &*h };
    let toc = psarc.psarc.toc();
    let idx = idx as usize;
    if idx >= toc.len() {
        return 0;
    }
    toc[idx].length
}

/// Extract all PSARC entries into `dest_dir`. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_psarc_extract_files(h: *mut PsarcHandle, dest_dir: *const c_char) -> i32 {
    if h.is_null() || dest_dir.is_null() {
        return -1;
    }
    let dir = unsafe { CStr::from_ptr(dest_dir) }.to_string_lossy();
    let psarc = unsafe { &mut *h };
    match psarc.psarc.extract_all(Path::new(dir.as_ref())) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Free a PSARC handle produced by `rs_psarc_open_file`.
#[no_mangle]
pub extern "C" fn rs_psarc_free(h: *mut PsarcHandle) {
    if !h.is_null() {
        unsafe { drop(Box::from_raw(h)) };
    }
}

// ─── Memory helpers ───────────────────────────────────────────────────────────

/// Free a C string allocated by this library (e.g. from `rs_psarc_manifest_get`).
#[no_mangle]
pub extern "C" fn rs_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}
