module Rocksmith2014.EOF.EOFTypes

/// Time signature discriminated union.
/// Mirrors `EOFTimeSignature` in `Rocksmith2014.EOF.EOFTypes` (F# reference).
type EOFTimeSignature =
    | ``TS 2 | 4``
    | ``TS 3 | 4``
    | ``TS 4 | 4``
    | ``TS 5 | 4``
    | ``TS 6 | 4``
    | CustomTS of nominator: uint * denominator: uint
