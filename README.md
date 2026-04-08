# Rocksmith2014.rs

[![Tests](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/test.yml/badge.svg)](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/test.yml)
[![Clippy & Format](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/lint.yml/badge.svg)](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/lint.yml)
[![Build Linux](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_linux.yml/badge.svg)](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_linux.yml)
[![Build macOS](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_macos.yml/badge.svg)](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_macos.yml)
[![Build Windows](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_windows.yml/badge.svg)](https://github.com/santzit/Rocksmith2014.rs/actions/workflows/build_windows.yml)

Rust library crates for [Rocksmith 2014](https://www.ubisoft.com/en-us/game/rocksmith) custom DLC.
A port of the [Rocksmith2014.NET](https://github.com/iminashi/Rocksmith2014.NET) libraries to Rust.

## Libraries

### Audio

Audio utilities: BS.1770 / ITU-R R128 LUFS loudness measurement, preview audio fade parameters and audio path helpers.

### Common

Common types and utilities shared across the other crates, including platform detection and zlib compression helpers.

### Conversion

Converts between the XML arrangement format and the binary SNG format used by the game engine.

### DLCProject

The `Arrangement` and `DLCProject` domain types, plus miscellaneous helpers needed for CDLC creation (DDS conversion, showlight generation, etc.).

### EOF

Creates [Editor on Fire](https://github.com/raynebc/editor-on-fire) project files from Rocksmith 2014 XML arrangements.

### FSharp Extensions

Ergonomic array, string, `Vec`, `Option`, and file utilities, ported from the F# standard-library extensions used in the .NET reference implementation.

### PSARC

Reads and writes PSARC archives — the proprietary container format used by Rocksmith 2014 DLC. The TOC is AES-256-CFB-128 encrypted; file data is split into 64 KB zlib-compressed blocks.

### SNG

Reads and writes SNG binary files — the compiled arrangement format consumed by the game engine. Payloads are AES-256-CTR encrypted (with separate PC and Mac keys) and zlib-compressed.

### XML

Parses and generates Rocksmith 2014 arrangement XML files: notes, chords, chord templates, anchors, phrases, phrase iterations, sections, events, hand shapes and arrangement metadata.

### XML Extension

Provides the `XmlEntity` enum (note or chord) and helpers for building sorted entity arrays from arrangement data.

### XML Processing

Checks arrangements for common errors and applies automatic improvements (anchor fixes, phrase cleanup, hand-shape adjustments, etc.).

## Roadmap / TODOs

### DD (Dynamic Difficulty)

Port the Dynamic Difficulty generator from [Rocksmith2014.NET](https://github.com/iminashi/Rocksmith2014.NET). This library generates multiple phrase difficulty levels from a fully-notated arrangement, allowing players to learn songs progressively.

### DD.Model

Port the DD data model types from [Rocksmith2014.NET](https://github.com/iminashi/Rocksmith2014.NET). These types describe the structure and parameters used by the Dynamic Difficulty generator (phrase levels, note densities, difficulty thresholds, etc.).

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable toolchain (1.70 or later recommended)

### Build all crates

```sh
cargo build --workspace
```

### Run all tests

```sh
cargo test --workspace
```

### Build a specific crate

```sh
cargo build -p rocksmith2014-psarc
```

## License

This project is licensed under the [MIT License](LICENSE).
