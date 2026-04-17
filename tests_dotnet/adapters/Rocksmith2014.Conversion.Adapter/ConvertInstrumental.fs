module Rocksmith2014.Conversion.ConvertInstrumental

open System.IO
open System.Runtime.InteropServices
open Rocksmith2014.SNG
open Rocksmith2014.XML

module private Ffi =
    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_arrangement_load(string path)

    [<DllImport("rocksmith2014_ffi")>]
    extern unit rs_arrangement_free(nativeint handle)

    [<DllImport("rocksmith2014_ffi")>]
    extern nativeint rs_sng_from_arrangement(nativeint arrHandle)

    [<DllImport("rocksmith2014_ffi")>]
    extern int rs_sng_to_xml_file(nativeint sngHandle, string path)

/// Convert an InstrumentalArrangement to SNG via Rust.
let xmlToSng (xml: InstrumentalArrangement) : SNG =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        xml.Save(temp)
        let arrHandle = Ffi.rs_arrangement_load(temp)
        if arrHandle = nativeint 0 then
            failwith "Rust failed to load arrangement XML"
        let sngHandle = Ffi.rs_sng_from_arrangement(arrHandle)
        Ffi.rs_arrangement_free(arrHandle)
        if sngHandle = nativeint 0 then
            failwith "Rust conversion failed: rs_sng_from_arrangement"
        new SNG(sngHandle)
    finally
        if File.Exists(temp) then File.Delete(temp)

/// Convert an SNG back to InstrumentalArrangement via Rust.
/// The `attr` parameter is accepted for API parity but currently ignored.
let sngToXml (_attr: _ option) (sng: SNG) : InstrumentalArrangement =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        let rc = Ffi.rs_sng_to_xml_file(sng.Handle, temp)
        if rc <> 0 then
            failwith "Rust conversion failed: rs_sng_to_xml_file"
        InstrumentalArrangement.Load(temp)
    finally
        if File.Exists(temp) then File.Delete(temp)
