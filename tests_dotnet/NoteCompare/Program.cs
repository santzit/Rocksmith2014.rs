// NoteCompare — Rust/.NET note-parity verifier.
//
// Usage:
//   NoteCompare <dlcDir> <rustCliBin>
//
// For every *.psarc in <dlcDir> the program:
//   1. Parses all XML arrangements with Rocksmith2014.NET (v3.5.0 reference).
//   2. Invokes <rustCliBin> psarc-notes <psarc> to get the Rust summary.
//   3. Diffs the two sorted TSV line sets and reports any discrepancy.
//
// Exit codes:
//   0 — all PSARCs match
//   1 — at least one mismatch (details printed to stdout)
//   2 — bad arguments or missing files

using System.Diagnostics;
using Rocksmith2014.PSARC;
using Rocksmith2014.XML;

if (args.Length != 2)
{
    Console.Error.WriteLine("usage: NoteCompare <dlcDir> <rustCliBin>");
    return 2;
}

var dlcDir    = args[0];
var rustBin   = args[1];
var allMatch  = true;

static ulong NoteChecksum(IEnumerable<Note> notes)
{
    ulong hash = 0xcbf29ce484222325;
    foreach (var row in notes
        .Select(n => $"{n.Time}:{n.String}:{n.Fret}:{n.Sustain}")
        .OrderBy(x => x, StringComparer.Ordinal))
    {
        foreach (var b in System.Text.Encoding.UTF8.GetBytes(row))
        {
            hash ^= b;
            hash *= 0x100000001b3;
        }
        hash ^= (byte)';';
        hash *= 0x100000001b3;
    }
    return hash;
}

static List<string> DotNetSummaryForPsarc(string psarcPath)
{
    var psarcName = Path.GetFileName(psarcPath);
    var lines = new List<string>();

    using var psarc = PSARC.OpenFile(psarcPath);
    foreach (var entry in psarc.Manifest
        .Where(x => x.EndsWith(".xml", StringComparison.Ordinal))
        .OrderBy(x => x))
    {
        try
        {
            // Extract XML entry to a temp file, load it, then delete.
            var tmp = Path.GetTempFileName();
            using (var stream = psarc.GetEntryStream(entry).GetAwaiter().GetResult())
            using (var outFile = File.Create(tmp))
            {
                stream.CopyTo(outFile);
            }

            var arr = InstrumentalArrangement.Load(tmp);
            File.Delete(tmp);

            foreach (var level in arr.Levels)
            {
                lines.Add(
                    $"{psarcName}\t{entry}\t{level.Difficulty}\t{level.Notes.Count}\t{NoteChecksum(level.Notes):x16}"
                );
            }
        }
        catch
        {
            // Skip non-instrumental XML entries (vocals, tones, etc.).
        }
    }

    lines.Sort(StringComparer.Ordinal);
    return lines;
}

static List<string> RustSummaryForPsarc(string rustBin, string psarcPath)
{
    var psi = new ProcessStartInfo(rustBin, $"psarc-notes \"{psarcPath}\"")
    {
        RedirectStandardOutput = true,
        RedirectStandardError  = true,
        UseShellExecute        = false,
    };

    using var proc = Process.Start(psi)
        ?? throw new InvalidOperationException("Failed to start Rust CLI.");

    var stdout = proc.StandardOutput.ReadToEnd();
    var stderr = proc.StandardError.ReadToEnd();
    proc.WaitForExit();

    if (proc.ExitCode != 0)
        throw new InvalidOperationException(
            $"Rust CLI exited {proc.ExitCode}: {stderr}");

    return stdout
        .Split('\n', StringSplitOptions.RemoveEmptyEntries)
        .Select(l => l.TrimEnd('\r'))
        .OrderBy(l => l, StringComparer.Ordinal)
        .ToList();
}

foreach (var psarcPath in Directory.EnumerateFiles(dlcDir, "*.psarc").OrderBy(x => x))
{
    Console.Write($"Checking {Path.GetFileName(psarcPath)} … ");

    List<string> dotnetLines;
    List<string> rustLines;

    try
    {
        dotnetLines = DotNetSummaryForPsarc(psarcPath);
        rustLines   = RustSummaryForPsarc(rustBin, psarcPath);
    }
    catch (Exception ex)
    {
        Console.WriteLine($"ERROR: {ex.Message}");
        allMatch = false;
        continue;
    }

    if (dotnetLines.SequenceEqual(rustLines))
    {
        Console.WriteLine($"OK ({dotnetLines.Count} level-rows)");
    }
    else
    {
        Console.WriteLine("MISMATCH");
        allMatch = false;

        var dotnetSet = dotnetLines.ToHashSet(StringComparer.Ordinal);
        var rustSet   = rustLines.ToHashSet(StringComparer.Ordinal);

        foreach (var line in dotnetLines.Except(rustLines, StringComparer.Ordinal))
            Console.WriteLine($"  - (dotnet only) {line}");

        foreach (var line in rustLines.Except(dotnetLines, StringComparer.Ordinal))
            Console.WriteLine($"  + (rust   only) {line}");
    }
}

if (allMatch)
{
    Console.WriteLine("\nAll PSARCs match — Rust and .NET note parsing is identical.");
    return 0;
}

Console.Error.WriteLine("\nFAILURE: note mismatch between Rust and .NET parsers.");
return 1;
