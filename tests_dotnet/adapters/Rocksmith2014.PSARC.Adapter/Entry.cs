namespace Rocksmith2014.PSARC;

/// <summary>Mirrors the F# Entry record: name digest + uncompressed length.</summary>
public sealed class Entry
{
    public ulong Length { get; }
    internal Entry(ulong length) => Length = length;
}
