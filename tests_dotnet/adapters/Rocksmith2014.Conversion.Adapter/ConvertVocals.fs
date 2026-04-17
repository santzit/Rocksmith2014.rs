module Rocksmith2014.Conversion.ConvertVocals

open System.IO
open System.Runtime.InteropServices
open Rocksmith2014.SNG
open Rocksmith2014.XML

module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_convert_vocals_xml_to_sng_default(string vocalsPath)

    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_convert_vocals_xml_to_sng_custom(string vocalsPath, string glyphsPath, string assetPath)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_convert_vocals_sng_to_xml_file(nativeint sngHandle, string outputPath)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_convert_extract_glyphs_file(nativeint sngHandle, string outputPath)

/// Convert a list of vocals and font option to SNG via Rust.
let xmlToSng (font: FontOption) (xml: System.Collections.Generic.List<Vocal>) : SNG =
    let temp1 = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        Vocals.Save(temp1, xml)
        let handle =
            match font with
            | DefaultFont ->
                Ffi.rs_convert_vocals_xml_to_sng_default(temp1)
            | CustomFont(glyphs, assetPath) ->
                let glyphTemp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
                try
                    glyphs.Save(glyphTemp)
                    Ffi.rs_convert_vocals_xml_to_sng_custom(temp1, glyphTemp, assetPath)
                finally
                    if File.Exists(glyphTemp) then File.Delete(glyphTemp)
        if handle = nativeint 0 then
            failwith "Rust conversion failed: rs_convert_vocals_xml_to_sng"
        new SNG(handle)
    finally
        if File.Exists(temp1) then File.Delete(temp1)

/// Convert SNG vocals back to a list of XML Vocal objects via Rust.
let sngToXml (sng: SNG) : System.Collections.Generic.List<Vocal> =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        let rc = Ffi.rs_convert_vocals_sng_to_xml_file(sng.Handle, temp)
        if rc <> 0 then
            failwith "Rust conversion failed: rs_convert_vocals_sng_to_xml_file"
        Vocals.Load(temp)
    finally
        if File.Exists(temp) then File.Delete(temp)

/// Extract glyph definitions from an SNG via Rust.
let extractGlyphData (sng: SNG) : GlyphDefinitions =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        let rc = Ffi.rs_convert_extract_glyphs_file(sng.Handle, temp)
        if rc <> 0 then
            failwith "Rust conversion failed: rs_convert_extract_glyphs_file"
        GlyphDefinitions.Load(temp)
    finally
        if File.Exists(temp) then File.Delete(temp)
