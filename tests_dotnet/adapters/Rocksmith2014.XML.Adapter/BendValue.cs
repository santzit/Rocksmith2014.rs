namespace Rocksmith2014.XML;

public struct BendValue : IEquatable<BendValue>
{
    public int Time { get; private set; }
    public float Step { get; private set; }

    public BendValue(int time, float step) { Time = time; Step = step; }

    public bool Equals(BendValue other) => Time == other.Time && Step == other.Step;
    public override bool Equals(object? obj) => obj is BendValue b && Equals(b);
    public override int GetHashCode() => (Time, Step).GetHashCode();
    public static bool operator ==(BendValue l, BendValue r) => l.Equals(r);
    public static bool operator !=(BendValue l, BendValue r) => !l.Equals(r);
}
