module Rocksmith2014.Audio.Volume

open System.Runtime.InteropServices

module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern double rs_audio_calculate_volume(string path)

/// Calculates a volume value using BS.1770 integrated loudness with -16 as reference value.
/// Mirrors `Volume.calculate` in the .NET reference.
let calculate (fileName: string) : float =
    let fullPath = System.IO.Path.GetFullPath(fileName)
    let v = Ffi.rs_audio_calculate_volume(fullPath)
    if System.Double.IsNaN(v) then
        failwithf "Rust audio volume calculation failed for '%s'" fileName
    v
