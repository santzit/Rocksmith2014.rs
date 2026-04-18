module Rocksmith2014.XML.Processing.ArrangementImprover

open System.IO
open Rocksmith2014.XML

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Copy all mutable collections from `src` into `dst` in-place.
let private copyBack (dst: InstrumentalArrangement) (src: InstrumentalArrangement) =
    dst.Ebeats.Clear();            dst.Ebeats.AddRange(src.Ebeats)
    dst.Phrases.Clear();           dst.Phrases.AddRange(src.Phrases)
    dst.PhraseIterations.Clear();  dst.PhraseIterations.AddRange(src.PhraseIterations)
    dst.Sections.Clear();          dst.Sections.AddRange(src.Sections)
    dst.Events.Clear();            dst.Events.AddRange(src.Events)
    dst.ChordTemplates.Clear();    dst.ChordTemplates.AddRange(src.ChordTemplates)
    dst.Levels.Clear();            dst.Levels.AddRange(src.Levels)
    // ToneInfo is a class with individual fields, not a ResizeArray.
    dst.Tones.BaseToneName <- src.Tones.BaseToneName
    for i in 0 .. src.Tones.Names.Length - 1 do
        dst.Tones.Names.[i] <- src.Tones.Names.[i]
    dst.Tones.Changes.Clear()
    dst.Tones.Changes.AddRange(src.Tones.Changes)

/// Run a Rust operation (expressed as `nativeint -> unit`) on `arr`.
/// Saves `arr` to a temp XML file, loads it into Rust, applies `rustOp`,
/// writes the result back to XML, reloads it, and copies the updated data
/// into the original arrangement in-place.
let private withRustImprove (rustOp: nativeint -> unit) (arr: InstrumentalArrangement) =
    let tempIn  = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    let tempOut = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        arr.Save(tempIn)
        let handle = RustFfi.rs_arrangement_load(tempIn)
        if handle = nativeint 0 then
            failwith "Rust: failed to load arrangement"
        try
            rustOp handle
            if RustFfi.rs_arrangement_save_xml(handle, tempOut) <> 0 then
                failwith "Rust: failed to save arrangement"
        finally
            RustFfi.rs_arrangement_free(handle)
        let updated = InstrumentalArrangement.Load(tempOut)
        copyBack arr updated
    finally
        if File.Exists(tempIn)  then File.Delete(tempIn)
        if File.Exists(tempOut) then File.Delete(tempOut)

// ── Public API ────────────────────────────────────────────────────────────────

/// Applies all improvements to the arrangement (Rust-backed).
let applyAll arrangement =
    withRustImprove RustFfi.rs_arrangement_apply_all arrangement

/// Applies the minimum set of improvements required for export (Rust-backed).
let applyMinimum arrangement =
    withRustImprove RustFfi.rs_arrangement_apply_minimum arrangement

/// Adds crowd events to the arrangement if it does not have them (Rust-backed).
let addCrowdEvents arrangement =
    withRustImprove RustFfi.rs_arrangement_add_crowd_events arrangement

/// Processes the chord template names.
/// Delegates to ChordNameProcessor to modify objects in-place, preserving
/// external references held by tests.
let processChordNames arrangement =
    ChordNameProcessor.improve arrangement

/// Removes beats that come after the audio has ended (Rust-backed).
let removeExtraBeats arrangement =
    withRustImprove RustFfi.rs_arrangement_remove_extra_beats arrangement

/// Applies all EOF fixes (Rust-backed).
let eofFixes arrangement =
    withRustImprove RustFfi.rs_arrangement_eof_fix_all arrangement

/// Moves phrases with a "mover" prefix to the Nth note (Rust-backed).
let movePhrases arrangement =
    withRustImprove RustFfi.rs_arrangement_move_phrases arrangement

/// Processes custom events (Rust-backed).
let processCustomEvents arrangement =
    withRustImprove RustFfi.rs_arrangement_process_custom_events arrangement

