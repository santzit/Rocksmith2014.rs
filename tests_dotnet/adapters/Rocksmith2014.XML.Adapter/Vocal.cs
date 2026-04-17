namespace Rocksmith2014.XML;

public sealed class Vocal
{
    public int Time { get; set; }
    public byte Note { get; set; }
    public int Length { get; set; }
    public string Lyric { get; set; } = string.Empty;

    public Vocal() { }
    public Vocal(int time, int length, string lyric, byte note = 60)
    { Time = time; Note = note; Length = length; Lyric = lyric; }
    public Vocal(Vocal other)
    { Time = other.Time; Note = other.Note; Length = other.Length; Lyric = other.Lyric; }
}
