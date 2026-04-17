namespace Rocksmith2014.Conversion

open Rocksmith2014.XML

/// Font selection for vocal SNG conversion.
/// Mirrors the F# `FontOption` discriminated union in the .NET reference.
type FontOption =
    | DefaultFont
    | CustomFont of glyphDefinitions: GlyphDefinitions * assetPath: string
