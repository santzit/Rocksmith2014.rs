using System.Runtime.InteropServices;

namespace Rocksmith2014.XML;

/// <summary>
/// Wraps Rust metadata parser via P/Invoke.
/// Failing tests reveal mismatches (e.g. field name mapping, string encoding).
/// </summary>
public sealed class MetaData : IDisposable
{
    private nint _handle;
    private bool _disposed;

    private MetaData(nint handle) => _handle = handle;

    /// <summary>Read metadata from an arrangement file (delegates to Rust).</summary>
    public static MetaData Read(string fileName)
    {
        var fullPath = Path.GetFullPath(fileName);
        var h = RustFfi.rs_metadata_read(fullPath);
        if (h == nint.Zero)
            throw new InvalidOperationException($"Rust parser failed to read metadata from '{fileName}'");
        return new MetaData(h);
    }

    public string? Title
        => RustFfi.PtrToString(RustFfi.rs_metadata_title(_handle));

    public float AverageTempo
        => RustFfi.rs_metadata_average_tempo(_handle);

    public string? ArtistNameSort
        => RustFfi.PtrToString(RustFfi.rs_metadata_artist_name_sort(_handle));

    public string? LastConversionDateTime
        => RustFfi.PtrToString(RustFfi.rs_metadata_last_conversion_datetime(_handle));

    public void Dispose()
    {
        if (!_disposed)
        {
            _disposed = true;
            RustFfi.rs_metadata_free(_handle);
            _handle = nint.Zero;
        }
    }
}
