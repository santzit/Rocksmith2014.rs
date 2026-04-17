using System.Runtime.InteropServices;

namespace Rocksmith2014.PSARC;

internal static partial class RustFfi
{
    private const string Lib = "rocksmith2014_ffi";

    [LibraryImport(Lib, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint rs_psarc_open_file(string path);

    [LibraryImport(Lib)]
    internal static partial int rs_psarc_manifest_count(nint handle);

    [LibraryImport(Lib)]
    internal static partial nint rs_psarc_manifest_get(nint handle, int idx);

    [LibraryImport(Lib)]
    internal static partial int rs_psarc_toc_count(nint handle);

    [LibraryImport(Lib)]
    internal static partial ulong rs_psarc_toc_entry_length(nint handle, int idx);

    [LibraryImport(Lib, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial int rs_psarc_extract_files(nint handle, string destDir);

    [LibraryImport(Lib)]
    internal static partial void rs_psarc_free(nint handle);

    [LibraryImport(Lib)]
    internal static partial void rs_free_string(nint ptr);

    internal static string? PtrToString(nint ptr) =>
        ptr == nint.Zero ? null : Marshal.PtrToStringUTF8(ptr);
}
