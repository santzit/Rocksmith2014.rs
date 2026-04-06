//! # rocksmith2014
//!
//! Rust library for reading Rocksmith 2014 PSARC, SNG and XML files,
//! with a C-compatible FFI suitable for use from game engines.
//!
//! ## Modules
//!
//! - [`psarc`] – PSARC archive reading (supports encrypted TOC and zlib-compressed entries).
//! - [`sng`] – SNG binary reading (AES-256-CTR decryption + zlib decompression).
//! - [`xml`] – Rocksmith 2014 instrumental arrangement XML parsing.
//! - [`ffi`] – C-compatible FFI layer (`rs2014_*` symbols).

pub mod error;
pub mod psarc;
pub mod sng;
pub mod xml;
pub mod ffi;

pub use error::{Error, Result};
pub use xml::utils;
