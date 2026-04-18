module Rocksmith2014.XML.Processing.ArrangementChecker

open System.IO
open System.Runtime.InteropServices
open Rocksmith2014.XML

// ── Issue conversion helpers ──────────────────────────────────────────────────

let private ptrToString (ptr: nativeint) =
    if ptr = nativeint 0 then None
    else Some (Marshal.PtrToStringUTF8(ptr))

let private issueTypeFromCode (code: string) (data: string option) : IssueType option =
    match code with
    | "I01" -> Some ApplauseEventWithoutEnd
    | "I02" -> data |> Option.map EventBetweenIntroApplause
    | "I03" -> Some NoteLinkedToChord
    | "I04" -> Some LinkNextMissingTargetNote
    | "I05" -> Some LinkNextSlideMismatch
    | "I06" -> Some LinkNextFretMismatch
    | "I07" -> Some LinkNextBendMismatch
    | "I08" -> Some IncorrectLinkNext
    | "I09" -> Some UnpitchedSlideWithLinkNext
    | "I10" -> Some PhraseChangeOnLinkNextNote
    | "I11" -> Some DoubleHarmonic
    | "I13" -> Some SeventhFretHarmonicWithSustain
    | "I14" -> Some MissingBendValue
    | "I15" -> Some ToneChangeOnNote
    | "I16" -> Some NoteInsideNoguitarSection
    | "I18" -> Some MissingLinkNextChordNotes
    | "I20" -> Some FingeringAnchorMismatch
    | "I21" -> Some AnchorInsideHandShape
    | "I22" -> Some AnchorInsideHandShapeAtPhraseBoundary
    | "I23" -> Some AnchorCloseToUnpitchedSlide
    | "I25" -> Some FirstPhraseNotEmpty
    | "I26" -> Some NoEndPhrase
    | "I27" -> Some PossiblyWrongChordFingering
    | "I28" -> Some BarreOverOpenStrings
    | "I29" -> Some MutedStringInNonMutedChord
    | "I30" -> Some MoreThan100Phrases
    | "I31" -> Some HopoIntoSameNote
    | "I32" -> Some FingerChangeDuringSlide
    | "I33" -> Some PositionShiftIntoPullOff
    | "I34" -> Some OverlappingBendValues
    | "I35" -> Some NaturalHarmonicWithBend
    | "I36" -> Some InvalidBassArrangementString
    | "I37" -> Some IncorrectMover1Phrase
    | "I38" -> Some FretNumberMoreThan24
    | "I39" -> Some NoteAfterSongEnd
    | "I40" -> Some TechniqueNoteWithoutSustain
    | "I41" -> Some LowBassTuningWithoutWorkaround
    | "I42" -> Some IncorrectLowBassTuningForTuningPitch
    | "V01" ->
        data |> Option.bind (fun d ->
            let parts = d.Split(':')
            if parts.Length = 2 then
                try Some (LyricWithInvalidChar(parts.[0].[0], System.Boolean.Parse(parts.[1])))
                with _ -> None
            else None)
    | "V02" -> data |> Option.map LyricTooLong
    | "V03" -> Some LyricsHaveNoLineBreaks
    | "S01" -> Some InvalidShowlights
    | _     -> None

let private issueFromRust (code: string) (time: int) (data: string option) : Issue option =
    issueTypeFromCode code data |> Option.map (fun t ->
        if time >= 0 then IssueWithTimeCode(t, time) else GeneralIssue t)

/// Collect all issues from a Rust IssueListHandle and free it.
let private collectAndFree (listHandle: nativeint) : Issue list =
    try
        let count = RustFfi.rs_issue_list_count(listHandle)
        [
            for i in 0 .. count - 1 do
                let code = RustFfi.rs_issue_list_code(listHandle, i) |> ptrToString |> Option.defaultValue ""
                let time = RustFfi.rs_issue_list_time(listHandle, i)
                let data = RustFfi.rs_issue_list_data(listHandle, i) |> ptrToString
                match issueFromRust code time data with
                | Some issue -> yield issue
                | None -> ()
        ]
    finally
        RustFfi.rs_issue_list_free(listHandle)

// ── Public API ────────────────────────────────────────────────────────────────

/// Runs all instrumental checks via Rust and returns a list of issues.
let checkInstrumental (arrangement: InstrumentalArrangement) =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        arrangement.Save(temp)
        let handle = RustFfi.rs_arrangement_load(temp)
        if handle = nativeint 0 then
            failwith "Rust: failed to load arrangement for checking"
        try
            let listHandle = RustFfi.rs_arrangement_check_instrumental(handle)
            collectAndFree listHandle
        finally
            RustFfi.rs_arrangement_free(handle)
    finally
        if File.Exists(temp) then File.Delete(temp)

/// Checks the vocals for issues via Rust.
let checkVocals (customFont: GlyphDefinitions option) (vocals: ResizeArray<Vocal>) =
    let tempVocals = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        Vocals.Save(tempVocals, vocals)
        match customFont with
        | None ->
            RustFfi.rs_vocals_check_file(tempVocals) |> collectAndFree
        | Some glyphs ->
            let tempGlyphs = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
            try
                glyphs.Save(tempGlyphs)
                RustFfi.rs_vocals_check_file_custom(tempVocals, tempGlyphs) |> collectAndFree
            finally
                if File.Exists(tempGlyphs) then File.Delete(tempGlyphs)
    finally
        if File.Exists(tempVocals) then File.Delete(tempVocals)

/// Checks that the show lights have at least one beam and one fog note via Rust.
let checkShowlights (showLights: ResizeArray<ShowLight>) =
    let temp = Path.ChangeExtension(Path.GetTempFileName(), ".xml")
    try
        ShowLights.Save(temp, showLights)
        RustFfi.rs_showlights_check_file(temp) |> collectAndFree
    finally
        if File.Exists(temp) then File.Delete(temp)

