using System.Runtime.InteropServices;

namespace Rocksmith2014.PSARC;

/// <summary>
/// Wraps Rust PSARC reader via P/Invoke.
///
/// ReadTests exercises Read / OpenFile / Manifest / TOC / ExtractFiles.
/// EditTests are excluded from CI (they require Rust write support).
/// Failing ReadTests reveal parsing mismatches between Rust and .NET.
/// </summary>
public sealed class PSARC : IDisposable, IAsyncDisposable
{
    private nint _handle;
    private string? _tempFile;   // non-null when opened from a Stream
    private bool _disposed;

    // Lazily populated.
    private IReadOnlyList<string>? _manifest;
    private IReadOnlyList<Entry>? _toc;

    private PSARC(nint handle, string? tempFile = null)
    {
        _handle = handle;
        _tempFile = tempFile;
    }

    // ── Factory methods ──────────────────────────────────────────────────────

    /// <summary>
    /// Read a PSARC from a stream (copied to a temp file for Rust to open).
    /// Mirrors <c>PSARC.Read(Stream)</c> in the .NET library.
    /// </summary>
    public static PSARC Read(Stream input)
    {
        var tmp = Path.GetTempFileName();
        using (var fs = File.Create(tmp))
            input.CopyTo(fs);

        var h = RustFfi.rs_psarc_open_file(tmp);
        if (h == nint.Zero)
        {
            File.Delete(tmp);
            throw new InvalidOperationException("Rust PSARC reader failed to open stream.");
        }
        return new PSARC(h, tmp);
    }

    /// <summary>
    /// Open a PSARC directly from a file path.
    /// Mirrors <c>PSARC.OpenFile(string)</c> in the .NET library.
    /// </summary>
    public static PSARC OpenFile(string path)
    {
        var fullPath = Path.GetFullPath(path);
        var h = RustFfi.rs_psarc_open_file(fullPath);
        if (h == nint.Zero)
            throw new InvalidOperationException($"Rust PSARC reader failed to open '{path}'.");
        return new PSARC(h);
    }

    // ── Properties ───────────────────────────────────────────────────────────

    /// <summary>Ordered list of entry names (the manifest).</summary>
    public IReadOnlyList<string> Manifest
    {
        get
        {
            if (_manifest is null)
            {
                var count = RustFfi.rs_psarc_manifest_count(_handle);
                var list = new List<string>(Math.Max(0, count));
                for (int i = 0; i < count; i++)
                {
                    var ptr = RustFfi.rs_psarc_manifest_get(_handle, i);
                    list.Add(RustFfi.PtrToString(ptr) ?? string.Empty);
                    RustFfi.rs_free_string(ptr);
                }
                _manifest = list;
            }
            return _manifest;
        }
    }

    /// <summary>Table of contents entries (one per file in the archive).</summary>
    public IReadOnlyList<Entry> TOC
    {
        get
        {
            if (_toc is null)
            {
                var count = RustFfi.rs_psarc_toc_count(_handle);
                var list = new List<Entry>(Math.Max(0, count));
                for (int i = 0; i < count; i++)
                    list.Add(new Entry(RustFfi.rs_psarc_toc_entry_length(_handle, i)));
                _toc = list;
            }
            return _toc;
        }
    }

    // ── Operations ───────────────────────────────────────────────────────────

    /// <summary>Extract all entries into <paramref name="baseDirectory"/>.</summary>
    public Task ExtractFiles(string baseDirectory)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        Directory.CreateDirectory(baseDirectory);
        var rc = RustFfi.rs_psarc_extract_files(_handle, baseDirectory);
        if (rc != 0)
            throw new IOException($"Rust ExtractFiles failed (rc={rc}).");
        return Task.CompletedTask;
    }

    // ── IDisposable ──────────────────────────────────────────────────────────

    public void Dispose()
    {
        if (!_disposed)
        {
            _disposed = true;
            RustFfi.rs_psarc_free(_handle);
            _handle = nint.Zero;
            if (_tempFile is not null)
            {
                try { File.Delete(_tempFile); } catch { /* best effort */ }
                _tempFile = null;
            }
        }
    }

    public ValueTask DisposeAsync()
    {
        Dispose();
        return ValueTask.CompletedTask;
    }
}
