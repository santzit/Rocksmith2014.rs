module Rocksmith2014.Audio.Utils

open System
open System.Runtime.InteropServices

module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern double rs_audio_get_length_ms(string path)

    [<DllImport("rocksmith2014_ffi", CharSet = CharSet.Ansi)>]
    extern nativeint rs_audio_create_preview_path(string path)

    [<DllImport("rocksmith2014_ffi")>]
    extern unit rs_free_string(nativeint ptr)

/// Returns the total length of the audio file as a TimeSpan.
/// Mirrors `Utils.getLength` in the .NET reference.
let getLength (fileName: string) : TimeSpan =
    let fullPath = IO.Path.GetFullPath(fileName)
    let ms = Ffi.rs_audio_get_length_ms(fullPath)
    if ms < 0.0 then
        failwithf "Rust audio get_length failed for '%s'" fileName
    TimeSpan.FromMilliseconds(ms)

/// Creates a path for the preview audio file from the given path.
/// Example: "some/path/file.ext" -> "some/path/file_preview.wav"
/// Mirrors `Utils.createPreviewAudioPath` in the .NET reference.
let createPreviewAudioPath (sourcePath: string) : string =
    let ptr = Ffi.rs_audio_create_preview_path(sourcePath)
    if ptr = nativeint 0 then ""
    else
        let s = Marshal.PtrToStringAnsi(ptr)
        Ffi.rs_free_string(ptr)
        if s = null then "" else s
