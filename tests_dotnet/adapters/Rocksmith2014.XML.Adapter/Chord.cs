namespace Rocksmith2014.XML;

/// <summary>
/// Bit values match .NET.  Rust uses a different layout (HIGH_DENSITY=bit1,
/// ACCENT=bit6, etc.) — mismatches will surface in conversion/SNG tests.
/// </summary>
[Flags]
public enum ChordMask : byte
{
    None         = 0,
    LinkNext     = 1 << 0,
    Accent       = 1 << 1,
    FretHandMute = 1 << 2,
    HighDensity  = 1 << 3,
    Ignore       = 1 << 4,
    PalmMute     = 1 << 5,
    Hopo         = 1 << 6,
}

public class Chord
{
    public int Time { get; set; }
    public short ChordId { get; set; }
    public ChordMask Mask { get; set; }
    public List<Note>? ChordNotes { get; set; }

    public Chord() { }
    public Chord(Chord other)
    {
        Time = other.Time;
        ChordId = other.ChordId;
        Mask = other.Mask;
        if (other.ChordNotes is not null)
            ChordNotes = other.ChordNotes.ConvertAll(n => new Note(n));
    }

    public bool HasChordNotes => ChordNotes is { Count: > 0 };

    private bool Get(ChordMask bit) => (Mask & bit) != 0;
    private void Set(ChordMask bit, bool value)
    { if (value) Mask |= bit; else Mask &= ~bit; }

    public bool IsLinkNext     { get => Get(ChordMask.LinkNext);     set => Set(ChordMask.LinkNext, value); }
    public bool IsAccent       { get => Get(ChordMask.Accent);       set => Set(ChordMask.Accent, value); }
    public bool IsFretHandMute { get => Get(ChordMask.FretHandMute); set => Set(ChordMask.FretHandMute, value); }
    public bool IsHighDensity  { get => Get(ChordMask.HighDensity);  set => Set(ChordMask.HighDensity, value); }
    public bool IsIgnore       { get => Get(ChordMask.Ignore);       set => Set(ChordMask.Ignore, value); }
    public bool IsPalmMute     { get => Get(ChordMask.PalmMute);     set => Set(ChordMask.PalmMute, value); }
    public bool IsHopo         { get => Get(ChordMask.Hopo);         set => Set(ChordMask.Hopo, value); }
}
