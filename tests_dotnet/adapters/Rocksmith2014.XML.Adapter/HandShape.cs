namespace Rocksmith2014.XML;

public sealed class HandShape
{
    public short ChordId { get; set; }
    public int StartTime { get; set; }
    public int EndTime { get; set; }
    public int Time => StartTime;

    public HandShape() { }
    public HandShape(short chordId, int startTime, int endTime)
    { ChordId = chordId; StartTime = startTime; EndTime = endTime; }
    public HandShape(HandShape other)
    { ChordId = other.ChordId; StartTime = other.StartTime; EndTime = other.EndTime; }
}
