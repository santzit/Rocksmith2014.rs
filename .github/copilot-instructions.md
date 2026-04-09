# Copilot Instructions for Rocksmith2014.rs

## Project Description

This is `Rocksmith2014.rs` — a Rust implementation of the [Rocksmith2014.NET](https://github.com/iminashi/Rocksmith2014.NET) library. The .NET project is written in F# and this is the Rust port.

## Knowledge Base Reference

Use `/Rocksmith2014.NET` folder/subproject as the primary knowledge base reference for understanding:
- The data structures and binary formats for SNG, PSARC, and other file formats
- The expected behavior of reading/writing operations
- The naming conventions for types and functions
- The strict same tests files, functions and parameters of .NET implementation, if the test FAIL is OK, report it and stop. We work on it later.
- Use same .NET file separation, eg.: `<filename>.fs` became `<filename>.rs`


The .NET project is in **F#** and this is the **Rust** implementation. Translate F# idioms to idiomatic Rust (e.g., F# discriminated unions → Rust enums, F# records → Rust structs, F# modules → Rust modules).

## Project Structure

```
rocksmith2014-audio/        — Audio processing (loudness, preview)
rocksmith2014-common/       — Shared types (Platform, AudioFile, binary I/O helpers)
  src/
    binary_readers.rs       — Binary reading utilities
    binary_writers.rs       — Binary writing utilities
    compression.rs          — zlib compression helpers
    json_options.rs         — JSON serialization configuration
    memory_stream_pool.rs   — Pooled Vec<u8> buffers
    platform.rs             — Platform enum (Pc/Mac)
    random_generator.rs     — Random number generation
    types.rs                — AudioFile and related types
rocksmith2014-conversion/   — XML ↔ SNG conversion
rocksmith2014-dlcproject/   — DLC project types
rocksmith2014-eof/          — EOF project writer
rocksmith2014-fsharp-extensions/ — F#-inspired extension traits
rocksmith2014-psarc/        — PSARC archive reader/writer
rocksmith2014-sng/          — SNG binary format reader/writer
  src/
    binary_helpers.rs       — Primitive read/write functions + SngRead/SngWrite traits
    cryptography.rs         — AES-CTR encryption/decryption
    types/                  — One file per SNG type (beat, note, level, …)
rocksmith2014-xml/          — Rocksmith XML types
rocksmith2014-xml-extension/ — XML extension utilities
rocksmith2014-xml-processing/ — XML processing (checker, improver)
tests/                      — Integration test crates mirroring .NET test structure
```

## Coding Conventions

- Use `thiserror` for error types
- Use `bitflags` for bitmask enums
- Use little-endian byte order for all binary I/O
- Keep public API backward compatible when refactoring
- Use same .NET file separation, eg.: `<filename>.fs` became `<filename>.rs`, do not mix files
- Use subproject directory naming `rocksmith2014-<subproject>`, eg.: `Rocksmith2014.Sng` became `rocksmith2014-sng`, `Rocksmith2014.Sng.Tests` became `rocksmith2014.sng-tests`, etc.
