namespace Rocksmith2014.SNG

open System.IO
open System.Runtime.InteropServices
open Rocksmith2014.Common

/// P/Invoke declarations into the Rust cdylib (librocksmith2014_ffi.so / .dll / .dylib).
module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_sng_read_unpacked(string path)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_sng_save_unpacked(nativeint handle, string path)

    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_sng_read_packed(string path, int platform)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_sng_save_packed(nativeint handle, string path, int platform)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_sng_level_count(nativeint handle)

    [<DllImport("rocksmith2014_ffi")>]
    extern unit rs_sng_free(nativeint handle)

    // ── Count getters ──────────────────────────────────────────────────────
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_beats_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_phrases_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_chord_templates_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_chord_notes_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_vocals_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_symbols_headers_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_symbols_textures_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_symbol_definitions_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_phrase_iterations_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_phrase_extra_info_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_new_linked_difficulties_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_events_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_tones_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_dnas_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_sections_count(nativeint h)

    // ── Texture properties ─────────────────────────────────────────────────
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_texture_width(nativeint h, int idx)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_texture_height(nativeint h, int idx)

    // ── MetaData ───────────────────────────────────────────────────────────
    [<DllImport("rocksmith2014_ffi")>] extern int16 rs_sng_meta_part(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern single rs_sng_meta_song_length(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_meta_tuning_count(nativeint h)
    [<DllImport("rocksmith2014_ffi")>] extern int16 rs_sng_meta_tuning(nativeint h, int idx)
    [<DllImport("rocksmith2014_ffi", CharSet = CharSet.Ansi)>]
    extern nativeint rs_sng_meta_last_conversion_datetime(nativeint h)

    // ── Vocals ─────────────────────────────────────────────────────────────
    [<DllImport("rocksmith2014_ffi", CharSet = CharSet.Ansi)>]
    extern nativeint rs_sng_vocal_lyric(nativeint h, int idx)
    [<DllImport("rocksmith2014_ffi")>] extern int rs_sng_vocal_note(nativeint h, int idx)
    [<DllImport("rocksmith2014_ffi")>] extern single rs_sng_vocal_time(nativeint h, int idx)
    [<DllImport("rocksmith2014_ffi")>] extern single rs_sng_vocal_length(nativeint h, int idx)

    // ── Free strings returned by Rust ──────────────────────────────────────
    [<DllImport("rocksmith2014_ffi")>]
    extern unit rs_free_string(nativeint ptr)

/// Helper to read a Rust-allocated CString and free it.
let private readAndFreeString (ptr: nativeint) : string =
    if ptr = nativeint 0 then ""
    else
        let s = Marshal.PtrToStringAnsi(ptr)
        Ffi.rs_free_string(ptr)
        if s = null then "" else s

/// Minimal placeholder — only Levels.Length is accessed by the kept test files.
type Level = { Difficulty: int }

/// A vocal entry with lyric, note, time and length.
type SngVocal = { Lyric: string; Note: int; Time: float32; Length: float32 }

/// Symbols texture with width and height.
type SngSymbolsTexture = { Width: int; Height: int }

/// SNG metadata subset used by conversion tests.
type SngMetaData = { Part: int16; LastConversionDateTime: string; Tuning: int16[]; SongLength: float32 }

/// Wraps the Rust SNG handle via P/Invoke.
/// Failing tests reveal behavioural mismatches between Rust and .NET.
type SNG internal (handle: nativeint) =
    let levelCount = Ffi.rs_sng_level_count(handle)
    let levels = Array.init (max 0 levelCount) (fun i -> { Difficulty = i })

    // ── Count arrays (unit placeholder) ────────────────────────────────────
    let beats              = Array.create (max 0 (Ffi.rs_sng_beats_count(handle))) ()
    let phrases            = Array.create (max 0 (Ffi.rs_sng_phrases_count(handle))) ()
    let chords             = Array.create (max 0 (Ffi.rs_sng_chord_templates_count(handle))) ()
    let chordNotes         = Array.create (max 0 (Ffi.rs_sng_chord_notes_count(handle))) ()
    let symbolsHeaders     = Array.create (max 0 (Ffi.rs_sng_symbols_headers_count(handle))) ()
    let symbolDefinitions  = Array.create (max 0 (Ffi.rs_sng_symbol_definitions_count(handle))) ()
    let phraseIterations   = Array.create (max 0 (Ffi.rs_sng_phrase_iterations_count(handle))) ()
    let phraseExtraInfo    = Array.create (max 0 (Ffi.rs_sng_phrase_extra_info_count(handle))) ()
    let newLinkedDiffs     = Array.create (max 0 (Ffi.rs_sng_new_linked_difficulties_count(handle))) ()
    let events             = Array.create (max 0 (Ffi.rs_sng_events_count(handle))) ()
    let tones              = Array.create (max 0 (Ffi.rs_sng_tones_count(handle))) ()
    let dnas               = Array.create (max 0 (Ffi.rs_sng_dnas_count(handle))) ()
    let sections           = Array.create (max 0 (Ffi.rs_sng_sections_count(handle))) ()

    // ── Symbols textures ────────────────────────────────────────────────────
    let symbolsTextures =
        let n = max 0 (Ffi.rs_sng_symbols_textures_count(handle))
        Array.init n (fun i ->
            { Width  = Ffi.rs_sng_texture_width(handle, i)
              Height = Ffi.rs_sng_texture_height(handle, i) })

    // ── Vocals ──────────────────────────────────────────────────────────────
    let vocals =
        let n = max 0 (Ffi.rs_sng_vocals_count(handle))
        Array.init n (fun i ->
            { Lyric  = readAndFreeString (Ffi.rs_sng_vocal_lyric(handle, i))
              Note   = Ffi.rs_sng_vocal_note(handle, i)
              Time   = Ffi.rs_sng_vocal_time(handle, i)
              Length = Ffi.rs_sng_vocal_length(handle, i) })

    // ── MetaData ─────────────────────────────────────────────────────────────
    let metaData =
        let tuningCount = max 0 (Ffi.rs_sng_meta_tuning_count(handle))
        { Part = Ffi.rs_sng_meta_part(handle)
          LastConversionDateTime = readAndFreeString (Ffi.rs_sng_meta_last_conversion_datetime(handle))
          Tuning = Array.init tuningCount (fun i -> Ffi.rs_sng_meta_tuning(handle, i))
          SongLength = Ffi.rs_sng_meta_song_length(handle) }

    member _.Levels : Level array              = levels
    member _.Beats : unit array                = beats
    member _.Phrases : unit array              = phrases
    member _.Chords : unit array               = chords
    member _.ChordNotes : unit array           = chordNotes
    member _.Vocals : SngVocal array           = vocals
    member _.SymbolsHeaders : unit array       = symbolsHeaders
    member _.SymbolsTextures : SngSymbolsTexture array = symbolsTextures
    member _.SymbolDefinitions : unit array    = symbolDefinitions
    member _.PhraseIterations : unit array     = phraseIterations
    member _.PhraseExtraInfo : unit array      = phraseExtraInfo
    member _.NewLinkedDifficulties : unit array = newLinkedDiffs
    member _.Events : unit array               = events
    member _.Tones : unit array                = tones
    member _.DNAs : unit array                 = dnas
    member _.Sections : unit array             = sections
    member _.MetaData : SngMetaData            = metaData
    member _.Handle : nativeint                = handle

    interface System.IDisposable with
        member _.Dispose() = Ffi.rs_sng_free(handle)

module SNG =
    let private platformId (p: Platform) =
        match p with
        | PC -> 0
        | Mac -> 1

    /// Read an unencrypted (unpacked) SNG file (delegates to Rust).
    let readUnpackedFile (path: string) : SNG =
        let fullPath = Path.GetFullPath(path)
        let h = Ffi.rs_sng_read_unpacked(fullPath)
        if h = nativeint 0 then
            failwithf "Rust SNG failed to read unpacked file '%s'" path
        new SNG(h)

    /// Save an unencrypted (unpacked) SNG file (delegates to Rust).
    let saveUnpackedFile (path: string) (sng: SNG) : unit =
        let fullPath = Path.GetFullPath(path)
        let rc = Ffi.rs_sng_save_unpacked(sng.Handle, fullPath)
        if rc <> 0 then
            failwithf "Rust SNG failed to save unpacked file '%s'" path

    /// Read an encrypted (packed) SNG file (delegates to Rust).
    let readPackedFile (path: string) (platform: Platform) : Async<SNG> =
        async {
            let fullPath = Path.GetFullPath(path)
            let h = Ffi.rs_sng_read_packed(fullPath, platformId platform)
            if h = nativeint 0 then
                failwithf "Rust SNG failed to read packed file '%s'" path
            return new SNG(h)
        }

    /// Save an encrypted (packed) SNG file (delegates to Rust).
    let savePackedFile (path: string) (platform: Platform) (sng: SNG) : Async<unit> =
        async {
            let fullPath = Path.GetFullPath(path)
            let rc = Ffi.rs_sng_save_packed(sng.Handle, fullPath, platformId platform)
            if rc <> 0 then
                failwithf "Rust SNG failed to save packed file '%s'" path
        }

