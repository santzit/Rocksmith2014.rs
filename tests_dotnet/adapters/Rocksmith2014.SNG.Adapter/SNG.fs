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

/// Minimal placeholder — only Levels.Length is accessed by the kept test files.
type Level = { Difficulty: int }

/// Wraps the Rust SNG handle via P/Invoke.
/// Failing tests reveal behavioural mismatches between Rust and .NET.
type SNG internal (handle: nativeint) =
    let levelCount = Ffi.rs_sng_level_count(handle)
    let levels = Array.init (max 0 levelCount) (fun i -> { Difficulty = i })

    member _.Levels : Level array = levels
    member _.Handle : nativeint = handle

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
