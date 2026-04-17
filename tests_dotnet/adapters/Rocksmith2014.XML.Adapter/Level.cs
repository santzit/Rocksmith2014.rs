namespace Rocksmith2014.XML;

/// <summary>
/// Stub level; the test only calls <c>.Count</c> on <c>arr.Levels</c>,
/// so this is sufficient.
/// </summary>
public sealed class Level
{
    public sbyte Difficulty { get; set; }
    public List<Note> Notes { get; set; } = [];
    public List<Chord> Chords { get; set; } = [];
    public List<Anchor> Anchors { get; set; } = [];
    public List<HandShape> HandShapes { get; set; } = [];
}
