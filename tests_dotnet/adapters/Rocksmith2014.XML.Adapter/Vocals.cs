using System.Text;
using System.Xml;

namespace Rocksmith2014.XML;

/// <summary>Pure C# XML I/O — same format as .NET Vocals.</summary>
public static class Vocals
{
    public static void Save(string fileName, List<Vocal> vocals)
    {
        var settings = new XmlWriterSettings
        {
            Indent = true,
            Encoding = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false)
        };
        using XmlWriter writer = XmlWriter.Create(fileName, settings);
        writer.WriteStartDocument();
        writer.WriteStartElement("vocals");
        writer.WriteAttributeString("count", vocals.Count.ToString());
        foreach (var v in vocals)
        {
            writer.WriteStartElement("vocal");
            writer.WriteAttributeString("time",   Utils.TimeCodeToString(v.Time));
            writer.WriteAttributeString("note",   v.Note.ToString());
            writer.WriteAttributeString("length", Utils.TimeCodeToString(v.Length));
            writer.WriteAttributeString("lyric",  v.Lyric);
            writer.WriteEndElement();
        }
        writer.WriteEndElement();
    }

    public static List<Vocal> Load(string fileName)
    {
        var settings = new XmlReaderSettings { IgnoreComments = true, IgnoreWhitespace = true };
        using XmlReader reader = XmlReader.Create(fileName, settings);
        reader.MoveToContent();

        var list = new List<Vocal>();
        if (reader.IsEmptyElement) { reader.ReadStartElement(); return list; }
        reader.ReadStartElement(); // <vocals>
        while (reader.NodeType != XmlNodeType.EndElement)
        {
            var v = new Vocal();
            for (int i = 0; i < reader.AttributeCount; i++)
            {
                reader.MoveToAttribute(i);
                switch (reader.Name)
                {
                    case "time":   v.Time   = Utils.TimeCodeFromFloatString(reader.Value); break;
                    case "note":   v.Note   = byte.Parse(reader.Value); break;
                    case "length": v.Length = Utils.TimeCodeFromFloatString(reader.Value); break;
                    case "lyric":  v.Lyric  = reader.Value; break;
                }
            }
            reader.MoveToElement();
            reader.ReadStartElement(); // <vocal ... />
            list.Add(v);
        }
        reader.ReadEndElement(); // </vocals>
        return list;
    }
}
