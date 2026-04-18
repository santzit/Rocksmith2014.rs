// P/Invoke declarations into the Rust cdylib (librocksmith2014_ffi.so / .dll / .dylib).
// The library name matches the [lib] name in rocksmith2014-ffi/Cargo.toml.
using System.Runtime.InteropServices;

namespace Rocksmith2014.XML;

internal static partial class RustFfi
{
    private const string Lib = "rocksmith2014_ffi";

    // ── InstrumentalArrangement ──────────────────────────────────────────────
    [LibraryImport(Lib, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint rs_arrangement_load(string path);

    [LibraryImport(Lib)]
    internal static partial int rs_arrangement_level_count(nint handle);

    [LibraryImport(Lib)]
    internal static partial void rs_arrangement_remove_dd(nint handle);

    [LibraryImport(Lib)]
    internal static partial void rs_arrangement_free(nint handle);

    // ── MetaData ─────────────────────────────────────────────────────────────
    [LibraryImport(Lib, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint rs_metadata_read(string path);

    [LibraryImport(Lib)]
    internal static partial nint rs_metadata_title(nint handle);

    [LibraryImport(Lib)]
    internal static partial float rs_metadata_average_tempo(nint handle);

    [LibraryImport(Lib)]
    internal static partial nint rs_metadata_artist_name_sort(nint handle);

    [LibraryImport(Lib)]
    internal static partial nint rs_metadata_last_conversion_datetime(nint handle);

    [LibraryImport(Lib)]
    internal static partial void rs_metadata_free(nint handle);

    // ── Memory ───────────────────────────────────────────────────────────────
    [LibraryImport(Lib)]
    internal static partial void rs_free_string(nint ptr);

    internal static string? PtrToString(nint ptr) =>
        ptr == nint.Zero ? null : Marshal.PtrToStringUTF8(ptr);
}
