namespace Rocksmith2014.XML;

/// <summary>
/// Wraps the Rust arrangement parser via P/Invoke.
/// Failing tests reveal behavioural mismatches between Rust and .NET.
/// </summary>
public sealed class InstrumentalArrangement : IDisposable
{
    private nint _handle;
    private bool _disposed;

    // Populated lazily from Rust after RemoveDD is called.
    private List<Level>? _levels;

    private InstrumentalArrangement(nint handle) => _handle = handle;

    /// <summary>Load and parse an arrangement from a file (delegates to Rust).</summary>
    public static InstrumentalArrangement Load(string fileName)
    {
        var fullPath = Path.GetFullPath(fileName);
        var h = RustFfi.rs_arrangement_load(fullPath);
        if (h == nint.Zero)
            throw new InvalidOperationException($"Rust parser failed to load '{fileName}'");
        return new InstrumentalArrangement(h);
    }

    /// <summary>Remove dynamic difficulty, keeping only the highest level.</summary>
    public Task RemoveDD()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        RustFfi.rs_arrangement_remove_dd(_handle);
        _levels = null; // invalidate cache
        return Task.CompletedTask;
    }

    /// <summary>Difficulty levels — count is sourced from Rust.</summary>
    public List<Level> Levels
    {
        get
        {
            if (_levels is null)
            {
                var count = RustFfi.rs_arrangement_level_count(_handle);
                _levels = Enumerable.Range(0, Math.Max(0, count))
                                    .Select(i => new Level { Difficulty = (sbyte)i })
                                    .ToList();
            }
            return _levels;
        }
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            _disposed = true;
            RustFfi.rs_arrangement_free(_handle);
            _handle = nint.Zero;
        }
    }
}
