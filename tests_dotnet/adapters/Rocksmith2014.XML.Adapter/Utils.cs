namespace Rocksmith2014.XML;

/// <summary>Mirrors Rocksmith2014.XML.Utils — pure C# implementation.</summary>
public static class Utils
{
    public static byte ParseBinary(string text)
    {
        char c = text[0];
        unchecked { c -= '0'; }
        return c >= 1 ? (byte)1 : (byte)0;
    }

    public static string TimeCodeToString(int timeCode)
    {
        string str = timeCode.ToString();
        return str.Length switch
        {
            1 => "0.00" + str,
            2 => "0.0" + str,
            3 => "0." + str,
            _ => str[..^3] + "." + str[^3..]
        };
    }

    // Called TimeCodeFromString in the test (wraps TimeCodeFromFloatString).
    public static int TimeCodeFromString(string input) => TimeCodeFromFloatString(input);

    public static int TimeCodeFromFloatString(string input)
    {
        int sep = input.IndexOf('.');
        if (sep == -1) return int.Parse(input) * 1000;

        Span<char> temp = stackalloc char[sep + 3];
        input.AsSpan(0, sep).CopyTo(temp);

        var decimals = input.AsSpan(sep + 1, Math.Min(input.Length - 1 - sep, 3));
        decimals.CopyTo(temp[sep..]);

        int i = temp.Length - 1;
        while (temp[i] == '\0') temp[i--] = '0';

        return int.Parse(temp);
    }
}
