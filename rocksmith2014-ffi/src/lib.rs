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

// C ABI functions must be non-unsafe for P/Invoke compatibility.
// Callers are responsible for passing valid pointers; null checks are at each entry point.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use rocksmith2014_psarc::Psarc;
use rocksmith2014_sng::{Platform as SngPlatform, Sng};
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
pub extern "C" fn rs_metadata_last_conversion_datetime(h: *const MetaDataHandle) -> *const c_char {
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

// ─── SNG ──────────────────────────────────────────────────────────────────────

pub struct SngHandle(Sng);

fn sng_platform(platform: i32) -> SngPlatform {
    if platform == 1 {
        SngPlatform::Mac
    } else {
        SngPlatform::Pc
    }
}

/// Read and parse an unencrypted (unpacked) SNG file. Returns null on error.
#[no_mangle]
pub extern "C" fn rs_sng_read_unpacked(path: *const c_char) -> *mut SngHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let data = match std::fs::read(Path::new(path.as_ref())) {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };
    match Sng::read(&data) {
        Ok(sng) => Box::into_raw(Box::new(SngHandle(sng))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Write an unencrypted (unpacked) SNG to a file. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_sng_save_unpacked(handle: *const SngHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let sng = &unsafe { &*handle }.0;
    match sng.write() {
        Ok(bytes) => match std::fs::write(Path::new(path.as_ref()), &bytes) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

/// Read and parse an encrypted (packed) SNG file. `platform`: 0=PC, 1=Mac. Returns null on error.
#[no_mangle]
pub extern "C" fn rs_sng_read_packed(path: *const c_char, platform: i32) -> *mut SngHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let data = match std::fs::read(Path::new(path.as_ref())) {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };
    match Sng::from_encrypted(&data, sng_platform(platform)) {
        Ok(sng) => Box::into_raw(Box::new(SngHandle(sng))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Write an encrypted (packed) SNG to a file. `platform`: 0=PC, 1=Mac. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_sng_save_packed(
    handle: *const SngHandle,
    path: *const c_char,
    platform: i32,
) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let sng = &unsafe { &*handle }.0;
    match sng.to_encrypted(sng_platform(platform)) {
        Ok(bytes) => match std::fs::write(Path::new(path.as_ref()), &bytes) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

/// Return the number of difficulty levels in the SNG. Returns -1 on null handle.
#[no_mangle]
pub extern "C" fn rs_sng_level_count(handle: *const SngHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    unsafe { &*handle }.0.levels.len() as i32
}

/// Free an SNG handle produced by `rs_sng_read_unpacked` or `rs_sng_read_packed`.
#[no_mangle]
pub extern "C" fn rs_sng_free(handle: *mut SngHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

// ─── SNG count getters ───────────────────────────────────────────────────────

macro_rules! sng_count {
    ($name:ident, $field:ident) => {
        #[no_mangle]
        pub extern "C" fn $name(h: *const SngHandle) -> i32 {
            if h.is_null() {
                return -1;
            }
            unsafe { &*h }.0.$field.len() as i32
        }
    };
}

sng_count!(rs_sng_beats_count, beats);
sng_count!(rs_sng_phrases_count, phrases);
sng_count!(rs_sng_chord_templates_count, chords);
sng_count!(rs_sng_chord_notes_count, chord_notes);
sng_count!(rs_sng_vocals_count, vocals);
sng_count!(rs_sng_symbols_headers_count, symbols_headers);
sng_count!(rs_sng_symbols_textures_count, symbols_textures);
sng_count!(rs_sng_symbol_definitions_count, symbol_definitions);
sng_count!(rs_sng_phrase_iterations_count, phrase_iterations);
sng_count!(rs_sng_phrase_extra_info_count, phrase_extra_info);
sng_count!(
    rs_sng_new_linked_difficulties_count,
    new_linked_difficulties
);
sng_count!(rs_sng_events_count, events);
sng_count!(rs_sng_tones_count, tones);
sng_count!(rs_sng_dnas_count, dnas);
sng_count!(rs_sng_sections_count, sections);

/// Width of the symbols texture at `idx`.  Returns -1 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_texture_width(h: *const SngHandle, idx: i32) -> i32 {
    if h.is_null() {
        return -1;
    }
    let sng = unsafe { &*h };
    sng.0
        .symbols_textures
        .get(idx as usize)
        .map(|t| t.width)
        .unwrap_or(-1)
}

/// Height of the symbols texture at `idx`.  Returns -1 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_texture_height(h: *const SngHandle, idx: i32) -> i32 {
    if h.is_null() {
        return -1;
    }
    let sng = unsafe { &*h };
    sng.0
        .symbols_textures
        .get(idx as usize)
        .map(|t| t.height)
        .unwrap_or(-1)
}

// ─── SNG MetaData getters ────────────────────────────────────────────────────

/// Arrangement part (int16).  Returns 0 on null.
#[no_mangle]
pub extern "C" fn rs_sng_meta_part(h: *const SngHandle) -> i16 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }.0.metadata.part
}

/// Song length in seconds (f32).  Returns 0 on null.
#[no_mangle]
pub extern "C" fn rs_sng_meta_song_length(h: *const SngHandle) -> f32 {
    if h.is_null() {
        return 0.0;
    }
    unsafe { &*h }.0.metadata.song_length
}

/// Number of tuning strings.  Returns 0 on null.
#[no_mangle]
pub extern "C" fn rs_sng_meta_tuning_count(h: *const SngHandle) -> i32 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }.0.metadata.tuning.len() as i32
}

/// Tuning value at `idx` (int16).  Returns 0 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_meta_tuning(h: *const SngHandle, idx: i32) -> i16 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }
        .0
        .metadata
        .tuning
        .get(idx as usize)
        .copied()
        .unwrap_or(0)
}

/// Last conversion datetime as a NUL-terminated UTF-8 string.
/// Caller must free with `rs_free_string`.
#[no_mangle]
pub extern "C" fn rs_sng_meta_last_conversion_datetime(h: *const SngHandle) -> *mut c_char {
    if h.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    let dt = &unsafe { &*h }.0.metadata.last_conversion_date_time;
    let end = dt.iter().position(|&b| b == 0).unwrap_or(dt.len());
    let s = String::from_utf8_lossy(&dt[..end]).into_owned();
    CString::new(s).unwrap_or_default().into_raw()
}

// ─── SNG Vocal getters ───────────────────────────────────────────────────────

/// Lyric at `idx` as a NUL-terminated UTF-8 string.  Caller must free with `rs_free_string`.
#[no_mangle]
pub extern "C" fn rs_sng_vocal_lyric(h: *const SngHandle, idx: i32) -> *mut c_char {
    if h.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    let sng = unsafe { &*h };
    let lyric = match sng.0.vocals.get(idx as usize) {
        Some(v) => {
            let end = v
                .lyric
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(v.lyric.len());
            String::from_utf8_lossy(&v.lyric[..end]).into_owned()
        }
        None => String::new(),
    };
    CString::new(lyric).unwrap_or_default().into_raw()
}

/// Note value at `idx` (i32).  Returns 0 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_vocal_note(h: *const SngHandle, idx: i32) -> i32 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }
        .0
        .vocals
        .get(idx as usize)
        .map(|v| v.note)
        .unwrap_or(0)
}

/// Time of vocal at `idx` in seconds (f32).  Returns 0 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_vocal_time(h: *const SngHandle, idx: i32) -> f32 {
    if h.is_null() {
        return 0.0;
    }
    unsafe { &*h }
        .0
        .vocals
        .get(idx as usize)
        .map(|v| v.time)
        .unwrap_or(0.0)
}

/// Length of vocal at `idx` in seconds (f32).  Returns 0 on invalid input.
#[no_mangle]
pub extern "C" fn rs_sng_vocal_length(h: *const SngHandle, idx: i32) -> f32 {
    if h.is_null() {
        return 0.0;
    }
    unsafe { &*h }
        .0
        .vocals
        .get(idx as usize)
        .map(|v| v.length)
        .unwrap_or(0.0)
}

// ─── Conversion — Instrumental ───────────────────────────────────────────────

/// Convert a loaded `InstrumentalArrangement` to SNG.  Returns null on null input.
#[no_mangle]
pub extern "C" fn rs_sng_from_arrangement(arr: *const ArrangementHandle) -> *mut SngHandle {
    if arr.is_null() {
        return std::ptr::null_mut();
    }
    let sng = rocksmith2014_conversion::xml_to_sng(&unsafe { &*arr }.0);
    Box::into_raw(Box::new(SngHandle(sng)))
}

/// Convert an SNG to InstrumentalArrangement XML and write to `path`.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_sng_to_xml_file(h: *const SngHandle, path: *const c_char) -> i32 {
    if h.is_null() || path.is_null() {
        return -1;
    }
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let arr = rocksmith2014_conversion::sng_to_xml_full(&unsafe { &*h }.0);
    match rocksmith2014_xml::write_file(&arr, Path::new(path_str.as_ref())) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// ─── Conversion — Vocals ─────────────────────────────────────────────────────

/// Convert a vocals XML file to SNG using the default font.
/// Returns a new SngHandle (free with `rs_sng_free`) or null on error.
#[no_mangle]
pub extern "C" fn rs_convert_vocals_xml_to_sng_default(
    vocals_path: *const c_char,
) -> *mut SngHandle {
    if vocals_path.is_null() {
        return std::ptr::null_mut();
    }
    let path_str = unsafe { CStr::from_ptr(vocals_path) }.to_string_lossy();
    let vocals = match rocksmith2014_xml::vocal::load(Path::new(path_str.as_ref())) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };
    let sng = rocksmith2014_conversion::xml_vocals_to_sng(
        rocksmith2014_conversion::FontOption::DefaultFont,
        &vocals,
    );
    Box::into_raw(Box::new(SngHandle(sng)))
}

/// Convert a vocals XML file to SNG using a custom glyph font.
/// `asset_path` is the DDS texture path embedded in the SNG.
/// Returns a new SngHandle (free with `rs_sng_free`) or null on error.
#[no_mangle]
pub extern "C" fn rs_convert_vocals_xml_to_sng_custom(
    vocals_path: *const c_char,
    glyphs_path: *const c_char,
    asset_path: *const c_char,
) -> *mut SngHandle {
    if vocals_path.is_null() || glyphs_path.is_null() || asset_path.is_null() {
        return std::ptr::null_mut();
    }
    let vocals_str = unsafe { CStr::from_ptr(vocals_path) }.to_string_lossy();
    let glyphs_str = unsafe { CStr::from_ptr(glyphs_path) }.to_string_lossy();
    let asset_str = unsafe { CStr::from_ptr(asset_path) }.to_string_lossy();

    let vocals = match rocksmith2014_xml::vocal::load(Path::new(vocals_str.as_ref())) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };
    let glyphs = match rocksmith2014_xml::GlyphDefinitions::load(Path::new(glyphs_str.as_ref())) {
        Ok(g) => g,
        Err(_) => return std::ptr::null_mut(),
    };
    let sng = rocksmith2014_conversion::xml_vocals_to_sng(
        rocksmith2014_conversion::FontOption::CustomFont(&glyphs, asset_str.as_ref()),
        &vocals,
    );
    Box::into_raw(Box::new(SngHandle(sng)))
}

/// Convert the vocals in an SNG to XML vocals format and write to `output_path`.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_convert_vocals_sng_to_xml_file(
    h: *const SngHandle,
    output_path: *const c_char,
) -> i32 {
    if h.is_null() || output_path.is_null() {
        return -1;
    }
    let path_str = unsafe { CStr::from_ptr(output_path) }.to_string_lossy();
    let xml_vocals = rocksmith2014_conversion::sng_vocals_to_xml(&unsafe { &*h }.0);
    match rocksmith2014_xml::vocal::save(Path::new(path_str.as_ref()), &xml_vocals) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Extract glyph definitions from an SNG and write to `output_path` as XML.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn rs_convert_extract_glyphs_file(
    h: *const SngHandle,
    output_path: *const c_char,
) -> i32 {
    if h.is_null() || output_path.is_null() {
        return -1;
    }
    let path_str = unsafe { CStr::from_ptr(output_path) }.to_string_lossy();
    let glyphs = rocksmith2014_conversion::extract_glyph_data(&unsafe { &*h }.0);
    match glyphs.save(Path::new(path_str.as_ref())) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// ─── Audio ───────────────────────────────────────────────────────────────────

/// Calculate the volume of an audio file (WAV/OGG/FLAC).
/// Returns the volume value (e.g. -0.2) or `f64::NAN` on error.
#[no_mangle]
pub extern "C" fn rs_audio_calculate_volume(path: *const c_char) -> f64 {
    if path.is_null() {
        return f64::NAN;
    }
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    rocksmith2014_audio::volume::calculate_from_file(Path::new(path_str.as_ref()))
        .unwrap_or(f64::NAN)
}

/// Get the duration of an audio file (WAV/OGG/FLAC) in milliseconds.
/// Returns -1.0 on error.
#[no_mangle]
pub extern "C" fn rs_audio_get_length_ms(path: *const c_char) -> f64 {
    if path.is_null() {
        return -1.0;
    }
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    match rocksmith2014_audio::utils::get_length(Path::new(path_str.as_ref())) {
        Ok(d) => d.as_secs_f64() * 1000.0,
        Err(_) => -1.0,
    }
}

/// Create a preview audio path string from a source path.
/// Returns a new NUL-terminated string; caller must free with `rs_free_string`.
#[no_mangle]
pub extern "C" fn rs_audio_create_preview_path(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let result = rocksmith2014_audio::utils::create_preview_audio_path(path_str.as_ref());
    CString::new(result).unwrap_or_default().into_raw()
}

// ─── EOF Helpers ─────────────────────────────────────────────────────────────

/// Get the index of the beat closest to `target_time` in milliseconds.
/// `times` and `measures` are parallel arrays of length `count`.
/// Returns 0 on null/empty input.
#[no_mangle]
pub extern "C" fn rs_eof_get_closest_beat(
    times: *const i32,
    measures: *const i16,
    count: i32,
    target_time: i32,
) -> i32 {
    if times.is_null() || measures.is_null() || count <= 0 {
        return 0;
    }
    let n = count as usize;
    let times_slice = unsafe { std::slice::from_raw_parts(times, n) };
    let measures_slice = unsafe { std::slice::from_raw_parts(measures, n) };
    let beats: Vec<rocksmith2014_xml::Ebeat> = times_slice
        .iter()
        .zip(measures_slice.iter())
        .map(|(&t, &m)| rocksmith2014_xml::Ebeat {
            time: t,
            measure: m,
        })
        .collect();
    rocksmith2014_eof::helpers::get_closest_beat(&beats, target_time) as i32
}

/// Try to parse a time signature from `text` (e.g. "TS:3/4").
/// If successful writes numerator/denominator to `n_out`/`d_out` and returns 1.
/// Returns 0 if the text is not a valid time signature.
#[no_mangle]
pub extern "C" fn rs_eof_try_parse_ts(
    text: *const c_char,
    n_out: *mut u32,
    d_out: *mut u32,
) -> i32 {
    if text.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    match rocksmith2014_eof::helpers::try_parse_time_signature(s.as_ref()) {
        Some((n, d)) => {
            if !n_out.is_null() {
                unsafe { *n_out = n };
            }
            if !d_out.is_null() {
                unsafe { *d_out = d };
            }
            1
        }
        None => 0,
    }
}

pub struct TsResultHandle(Vec<(i32, rocksmith2014_eof::types::TimeSignature)>);

/// Infer time signatures from an array of beats.
/// Returns a TsResultHandle; free with `rs_eof_ts_free`.
#[no_mangle]
pub extern "C" fn rs_eof_infer_time_signatures(
    times: *const i32,
    measures: *const i16,
    count: i32,
) -> *mut TsResultHandle {
    if times.is_null() || measures.is_null() || count <= 0 {
        return Box::into_raw(Box::new(TsResultHandle(Vec::new())));
    }
    let n = count as usize;
    let times_slice = unsafe { std::slice::from_raw_parts(times, n) };
    let measures_slice = unsafe { std::slice::from_raw_parts(measures, n) };
    let beats: Vec<rocksmith2014_xml::Ebeat> = times_slice
        .iter()
        .zip(measures_slice.iter())
        .map(|(&t, &m)| rocksmith2014_xml::Ebeat {
            time: t,
            measure: m,
        })
        .collect();
    let result = rocksmith2014_eof::helpers::infer_time_signatures(&beats);
    Box::into_raw(Box::new(TsResultHandle(result)))
}

/// Number of time signature results.
#[no_mangle]
pub extern "C" fn rs_eof_ts_count(h: *const TsResultHandle) -> i32 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }.0.len() as i32
}

/// Time (ms) of the time signature result at `idx`.
#[no_mangle]
pub extern "C" fn rs_eof_ts_time(h: *const TsResultHandle, idx: i32) -> i32 {
    if h.is_null() {
        return 0;
    }
    unsafe { &*h }
        .0
        .get(idx as usize)
        .map(|(t, _)| *t)
        .unwrap_or(0)
}

/// Tag for the time signature at `idx`:
/// 0=TS2/4, 1=TS3/4, 2=TS4/4, 3=TS5/4, 4=TS6/4, 5=Custom.
#[no_mangle]
pub extern "C" fn rs_eof_ts_tag(h: *const TsResultHandle, idx: i32) -> u8 {
    if h.is_null() {
        return 0;
    }
    match unsafe { &*h }.0.get(idx as usize) {
        Some((_, ts)) => match ts {
            rocksmith2014_eof::types::TimeSignature::TS2_4 => 0,
            rocksmith2014_eof::types::TimeSignature::TS3_4 => 1,
            rocksmith2014_eof::types::TimeSignature::TS4_4 => 2,
            rocksmith2014_eof::types::TimeSignature::TS5_4 => 3,
            rocksmith2014_eof::types::TimeSignature::TS6_4 => 4,
            rocksmith2014_eof::types::TimeSignature::Custom(_, _) => 5,
        },
        None => 0,
    }
}

/// Numerator of a Custom time signature at `idx`.  Returns 0 for non-Custom.
#[no_mangle]
pub extern "C" fn rs_eof_ts_num(h: *const TsResultHandle, idx: i32) -> u32 {
    if h.is_null() {
        return 0;
    }
    match unsafe { &*h }.0.get(idx as usize) {
        Some((_, rocksmith2014_eof::types::TimeSignature::Custom(n, _))) => *n,
        _ => 0,
    }
}

/// Denominator of a Custom time signature at `idx`.  Returns 0 for non-Custom.
#[no_mangle]
pub extern "C" fn rs_eof_ts_den(h: *const TsResultHandle, idx: i32) -> u32 {
    if h.is_null() {
        return 0;
    }
    match unsafe { &*h }.0.get(idx as usize) {
        Some((_, rocksmith2014_eof::types::TimeSignature::Custom(_, d))) => *d,
        _ => 0,
    }
}

/// Free a TsResultHandle.
#[no_mangle]
pub extern "C" fn rs_eof_ts_free(h: *mut TsResultHandle) {
    if !h.is_null() {
        unsafe { drop(Box::from_raw(h)) };
    }
}
