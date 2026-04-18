namespace Rocksmith2014.XML;

public sealed class Anchor : IEquatable<Anchor>
{
    public sbyte Fret { get; set; }
    public int Time { get; set; }
    public sbyte Width { get; set; }

    public Anchor() { }
    public Anchor(sbyte fret, int time, sbyte width = 4) { Fret = fret; Time = time; Width = width; }
    public Anchor(Anchor other) { Fret = other.Fret; Time = other.Time; Width = other.Width; }

    public bool Equals(Anchor? other) =>
        other is not null && Fret == other.Fret && Width == other.Width && Time == other.Time;
    public override bool Equals(object? obj) => obj is Anchor a && Equals(a);
    public override int GetHashCode() => (Time, Fret, Width).GetHashCode();
    public static bool operator ==(Anchor? l, Anchor? r) => l is null ? r is null : l.Equals(r);
    public static bool operator !=(Anchor? l, Anchor? r) => !(l == r);
}
