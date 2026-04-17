module Rocksmith2014.EOF.Helpers

open System.Runtime.InteropServices
open Rocksmith2014.XML
open Rocksmith2014.EOF.EOFTypes

module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_eof_get_closest_beat(
        [<In>] int[] times,
        [<In>] int16[] measures,
        int count,
        int targetTime)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_eof_try_parse_ts(
        string text,
        [<Out>] uint32& nOut,
        [<Out>] uint32& dOut)

    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_eof_infer_time_signatures(
        [<In>] int[] times,
        [<In>] int16[] measures,
        int count)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_eof_ts_count(nativeint h)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_eof_ts_time(nativeint h, int idx)

    // 0=TS2/4, 1=TS3/4, 2=TS4/4, 3=TS5/4, 4=TS6/4, 5=Custom
    [<DllImport("rocksmith2014_ffi")>]
    extern byte rs_eof_ts_tag(nativeint h, int idx)

    [<DllImport("rocksmith2014_ffi")>]
    extern uint32 rs_eof_ts_num(nativeint h, int idx)

    [<DllImport("rocksmith2014_ffi")>]
    extern uint32 rs_eof_ts_den(nativeint h, int idx)

    [<DllImport("rocksmith2014_ffi")>]
    extern unit rs_eof_ts_free(nativeint h)

/// Convert an array of Ebeats to parallel int[]/int16[] arrays for FFI.
let private toArrays (beats: Ebeat seq) =
    let arr = beats |> Seq.toArray
    let times    = arr |> Array.map (fun b -> b.Time)
    let measures = arr |> Array.map (fun b -> b.Measure)
    times, measures

/// Tag byte → EOFTimeSignature (Custom variant uses num/den from handle).
let private decodeTag (h: nativeint) (idx: int) : EOFTimeSignature =
    match Ffi.rs_eof_ts_tag(h, idx) with
    | 0uy -> ``TS 2 | 4``
    | 1uy -> ``TS 3 | 4``
    | 2uy -> ``TS 4 | 4``
    | 3uy -> ``TS 5 | 4``
    | 4uy -> ``TS 6 | 4``
    | _   -> CustomTS(Ffi.rs_eof_ts_num(h, idx), Ffi.rs_eof_ts_den(h, idx))

/// Infer time signatures from a sequence of beats.
/// Mirrors `Helpers.inferTimeSignatures` in the F# reference.
let inferTimeSignatures (beats: Ebeat seq) : seq<int * EOFTimeSignature> =
    let times, measures = toArrays beats
    let count = times.Length
    let h = Ffi.rs_eof_infer_time_signatures(times, measures, count)
    try
        let n = Ffi.rs_eof_ts_count(h)
        seq {
            for i in 0 .. n - 1 do
                yield (Ffi.rs_eof_ts_time(h, i), decodeTag h i)
        }
        |> Seq.toList  // materialize before freeing handle
        |> List.toSeq
    finally
        Ffi.rs_eof_ts_free(h)

/// Try to parse a time signature string (e.g. "TS:4/4").
/// Returns Some (numerator, denominator) on success, None otherwise.
/// Mirrors `Helpers.tryParseTimeSignature` in the F# reference.
let tryParseTimeSignature (text: string) : (uint * uint) option =
    let mutable n = 0u
    let mutable d = 0u
    let rc = Ffi.rs_eof_try_parse_ts(text, &n, &d)
    if rc = 1 then Some (n, d)
    else None

/// Find the index of the beat closest to `time` (in milliseconds).
/// Mirrors `Helpers.getClosestBeat` in the F# reference.
let getClosestBeat (beats: Ebeat[]) (time: int) : int =
    let times    = beats |> Array.map (fun b -> b.Time)
    let measures = beats |> Array.map (fun b -> b.Measure)
    Ffi.rs_eof_get_closest_beat(times, measures, times.Length, time)
