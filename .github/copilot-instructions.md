# Copilot Instructions for Rocksmith2014.rs

## Project Description

This is `Rocksmith2014.rs` — a Rust implementation of the [Rocksmith2014.NET](https://github.com/iminashi/Rocksmith2014.NET) library. 
The `.NET` project is written in `F#` (tests use `C#`) and this is the `Rust` port.


## Project implementation status
Create and update an table on project README.md file with the submodule, number of tests, passed, ignored, failed.

example:
|Submodule|Tests|Passed|Ignored|Failed|
|------|------|------|------|------|
|Audio|20|18|0|2|
|Common|32|4|2|26|

Update this table at each commit/unit tests

## Knowledge Base Reference

Use `/Rocksmith2014.NET` folder/subproject as the primary knowledge base reference for understanding:
- The data structures and binary formats for SNG, PSARC, and other file formats
- The expected behavior of reading/writing operations
- The naming conventions for types and functions
- The strict same tests files, functions and parameters of .NET implementation, if the test FAIL, report it (reply/comment) and stop the session. We work on it later.
- Use same .NET file separation, eg.: `<filename>.fs` became `<filename>.rs`
- Use [ww2ogg](https://github.com/hcs64/ww2ogg) for Wwise cli (decoding only)
- Use same .NET file separation, eg.: `<filename>.fs` became `<filename>.rs`, do not mix files
- Use subproject directory naming `rocksmith2014-<subproject>`, eg.: `Rocksmith2014.Sng` became `rocksmith2014-sng`, `Rocksmith2014.Sng.Tests` became `rocksmith2014.sng-tests`, etc.
- Use same public methods, functions names
- Use same data, structures, enum
- Keep same file and directory path, eg. `src/Rocksmith2014.Common/Profile` should became `src/rocksmith2014-common/profile`
- Do not use generic/consolidated lib.rs, use separated files segregation like .NET implementation.

The .NET project is in **F#** and this is the **Rust** implementation. Translate F# idioms to idiomatic Rust (e.g., F# discriminated unions → Rust enums, F# records → Rust structs, F# modules → Rust modules).

Note: we can mark encode and japanese tests as `ignore`, we will not work on audio encoding or use japanese for now.


## Project Structure

```
rocksmith2014-audio/        — Audio processing (loudness, preview)
rocksmith2014-common/       — Shared types (Platform, AudioFile, binary I/O helpers)
rocksmith2014-conversion/   — XML ↔ SNG conversion
rocksmith2014-dlcproject/   — DLC project types
rocksmith2014-eof/          — EOF project writer
rocksmith2014-fsharp-extensions/ — F#-inspired extension traits
rocksmith2014-psarc/        — PSARC archive reader/writer
rocksmith2014-sng/          — SNG binary format reader/writer
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
