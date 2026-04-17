using System.Text;
using System.Xml;

namespace Rocksmith2014.XML;

/// <summary>Pure C# XML I/O — same format as .NET ShowLights.</summary>
public static class ShowLights
{
    public static void Save(string fileName, List<ShowLight> showLights)
    {
        var settings = new XmlWriterSettings
        {
            Indent = true,
            Encoding = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false)
        };
        using XmlWriter writer = XmlWriter.Create(fileName, settings);
        writer.WriteStartDocument();
        writer.WriteStartElement("showlights");
        writer.WriteAttributeString("count", showLights.Count.ToString());
        foreach (var sl in showLights)
        {
            writer.WriteStartElement("showlight");
            writer.WriteAttributeString("time", Utils.TimeCodeToString(sl.Time));
            writer.WriteAttributeString("note", sl.Note.ToString());
            writer.WriteEndElement();
        }
        writer.WriteEndElement();
    }

    public static List<ShowLight> Load(string fileName)
    {
        var settings = new XmlReaderSettings { IgnoreComments = true, IgnoreWhitespace = true };
        using XmlReader reader = XmlReader.Create(fileName, settings);
        reader.MoveToContent();

        var list = new List<ShowLight>();
        if (reader.IsEmptyElement) { reader.ReadStartElement(); return list; }
        reader.ReadStartElement(); // <showlights>
        while (reader.NodeType != XmlNodeType.EndElement)
        {
            var sl = new ShowLight();
            for (int i = 0; i < reader.AttributeCount; i++)
            {
                reader.MoveToAttribute(i);
                if (reader.Name == "time")       sl.Time = Utils.TimeCodeFromFloatString(reader.Value);
                else if (reader.Name == "note")  sl.Note = byte.Parse(reader.Value);
            }
            reader.MoveToElement();
            reader.ReadStartElement(); // <showlight ... />
            list.Add(sl);
        }
        reader.ReadEndElement(); // </showlights>
        return list;
    }
}
