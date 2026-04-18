namespace Rocksmith2014.XML;

public sealed class ShowLight
{
    public const byte FogMin  = 24;
    public const byte FogMax  = 35;
    public const byte BeamOff = 42;
    public const byte BeamMin = 48;
    public const byte BeamMax = 59;
    public const byte LasersOff = 66;
    public const byte LasersOn  = 67;

    public int Time { get; set; }
    public byte Note { get; set; }

    public ShowLight() { }
    public ShowLight(int time, byte note) { Time = time; Note = note; }

    public bool IsFog()  => Note >= FogMin && Note <= FogMax;
    public bool IsBeam() => (Note >= BeamMin && Note <= BeamMax) || Note == BeamOff;
}
