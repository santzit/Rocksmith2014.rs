module internal Rocksmith2014.XML.Processing.RustFfi

open System.Runtime.InteropServices

[<Literal>]
let private Lib = "rocksmith2014_ffi"

// ── Arrangement I/O ──────────────────────────────────────────────────────────

[<DllImport(Lib)>]
extern nativeint rs_arrangement_load(string path)

[<DllImport(Lib)>]
extern unit rs_arrangement_free(nativeint handle)

[<DllImport(Lib)>]
extern int rs_arrangement_save_xml(nativeint handle, string path)

// ── Improvers ─────────────────────────────────────────────────────────────────

[<DllImport(Lib)>]
extern unit rs_arrangement_apply_all(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_apply_minimum(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_validate_phrase_names(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_add_ignores(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_fix_link_nexts(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_remove_overlapping_bend_values(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_remove_muted_notes_from_chords(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_remove_redundant_anchors(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_add_crowd_events(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_process_chord_names(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_remove_extra_beats(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_move_anchors(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_move_phrases(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_process_custom_events(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_lengthen_handshapes(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_shorten_handshapes(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_eof_fix_all(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_eof_fix_chord_notes(nativeint handle)

[<DllImport(Lib)>]
extern unit rs_arrangement_eof_fix_phrase_start_anchors(nativeint handle)

// ── Checkers ──────────────────────────────────────────────────────────────────

[<DllImport(Lib)>]
extern nativeint rs_arrangement_check_instrumental(nativeint handle)

[<DllImport(Lib)>]
extern nativeint rs_showlights_check_file(string path)

[<DllImport(Lib)>]
extern nativeint rs_vocals_check_file(string path)

[<DllImport(Lib)>]
extern nativeint rs_vocals_check_file_custom(string vocalsPath, string glyphsPath)

// ── Issue list accessors ──────────────────────────────────────────────────────

[<DllImport(Lib)>]
extern int rs_issue_list_count(nativeint handle)

[<DllImport(Lib)>]
extern nativeint rs_issue_list_code(nativeint handle, int idx)

[<DllImport(Lib)>]
extern int rs_issue_list_time(nativeint handle, int idx)

[<DllImport(Lib)>]
extern nativeint rs_issue_list_data(nativeint handle, int idx)

[<DllImport(Lib)>]
extern unit rs_issue_list_free(nativeint handle)
