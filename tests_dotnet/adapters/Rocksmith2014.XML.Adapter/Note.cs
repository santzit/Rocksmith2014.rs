namespace Rocksmith2014.XML;

/// <summary>
/// Bit values match the .NET implementation exactly so that pure in-memory
/// mask tests pass.  Mismatches with Rust's ChordMask bit layout will surface
/// in file-based round-trip and conversion tests.
/// </summary>
[Flags]
public enum NoteMask : ushort
{
    None         = 0,
    LinkNext     = 1 << 0,
    Accent       = 1 << 1,
    HammerOn     = 1 << 2,
    Harmonic     = 1 << 3,
    Ignore       = 1 << 4,
    FretHandMute = 1 << 5,
    PalmMute     = 1 << 6,
    PullOff      = 1 << 7,
    Tremolo      = 1 << 8,
    PinchHarmonic = 1 << 9,
    PickDirection = 1 << 10,
    Slap         = 1 << 11,
    Pluck        = 1 << 12,
    RightHand    = 1 << 13,
}

public class Note
{
    public NoteMask Mask { get; set; }
    public int Time { get; set; }
    public sbyte String { get; set; }
    public sbyte Fret { get; set; }
    public int Sustain { get; set; }
    public sbyte LeftHand { get; set; } = -1;
    public sbyte SlideTo { get; set; } = -1;
    public sbyte SlideUnpitchTo { get; set; } = -1;
    public byte Tap { get; set; }
    public byte Vibrato { get; set; }
    public float MaxBend { get; set; }
    public List<BendValue>? BendValues { get; set; }

    public Note() { }
    public Note(Note other)
    {
        Mask = other.Mask;
        Time = other.Time;
        String = other.String;
        Fret = other.Fret;
        Sustain = other.Sustain;
        LeftHand = other.LeftHand;
        SlideTo = other.SlideTo;
        SlideUnpitchTo = other.SlideUnpitchTo;
        Tap = other.Tap;
        Vibrato = other.Vibrato;
        MaxBend = other.MaxBend;
        if (other.BendValues is not null)
            BendValues = new List<BendValue>(other.BendValues);
    }

    // ── mask convenience ────────────────────────────────────────────────────
    private bool Get(NoteMask bit) => (Mask & bit) != 0;
    private void Set(NoteMask bit, bool value)
    { if (value) Mask |= bit; else Mask &= ~bit; }

    public bool IsLinkNext     { get => Get(NoteMask.LinkNext);     set => Set(NoteMask.LinkNext, value); }
    public bool IsAccent       { get => Get(NoteMask.Accent);       set => Set(NoteMask.Accent, value); }
    public bool IsHammerOn     { get => Get(NoteMask.HammerOn);     set => Set(NoteMask.HammerOn, value); }
    public bool IsHarmonic     { get => Get(NoteMask.Harmonic);     set => Set(NoteMask.Harmonic, value); }
    public bool IsIgnore       { get => Get(NoteMask.Ignore);       set => Set(NoteMask.Ignore, value); }
    public bool IsFretHandMute { get => Get(NoteMask.FretHandMute); set => Set(NoteMask.FretHandMute, value); }
    public bool IsPalmMute     { get => Get(NoteMask.PalmMute);     set => Set(NoteMask.PalmMute, value); }
    public bool IsPullOff      { get => Get(NoteMask.PullOff);      set => Set(NoteMask.PullOff, value); }
    public bool IsTremolo      { get => Get(NoteMask.Tremolo);      set => Set(NoteMask.Tremolo, value); }
    public bool IsPinchHarmonic{ get => Get(NoteMask.PinchHarmonic);set => Set(NoteMask.PinchHarmonic, value); }
    public bool IsSlap         { get => Get(NoteMask.Slap);         set => Set(NoteMask.Slap, value); }
    public bool IsPluck        { get => Get(NoteMask.Pluck);        set => Set(NoteMask.Pluck, value); }
    public bool IsRightHand    { get => Get(NoteMask.RightHand);    set => Set(NoteMask.RightHand, value); }

    // ── other convenience ───────────────────────────────────────────────────
    public bool IsBend          => BendValues is { Count: > 0 };
    public bool IsSlide         => SlideTo != -1;
    public bool IsUnpitchedSlide => SlideUnpitchTo != -1;
    public bool IsVibrato       => Vibrato != 0;
    public bool IsTap           => Tap != 0;
}
