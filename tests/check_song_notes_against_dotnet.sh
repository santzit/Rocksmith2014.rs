#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOTNET_DIR="$REPO_ROOT/tests/rocksmith2014-net"
DLC_DIR="$REPO_ROOT/tests/DLC"
RUST_SUMMARY="/tmp/rust-dlc-notes-summary.tsv"
DOTNET_SUMMARY="/tmp/dotnet-dlc-notes-summary.tsv"
DOTNET_APP_DIR="/tmp/dotnet-dlc-note-checker"

if [ ! -d "$DOTNET_DIR/.git" ]; then
  git clone https://github.com/iminashi/Rocksmith2014.NET "$DOTNET_DIR"
fi

git -C "$DOTNET_DIR" fetch --tags origin
git -C "$DOTNET_DIR" checkout --quiet tags/v3.5.0

cd "$REPO_ROOT"
RUST_DLC_NOTE_SUMMARY_PATH="$RUST_SUMMARY" \
  cargo test -p rocksmith2014-audio-tests --test DlcNotesReportTests -- --nocapture

dotnet build "$DOTNET_DIR/src/Rocksmith2014.PSARC/Rocksmith2014.PSARC.fsproj" -c Release --nologo >/dev/null
dotnet build "$DOTNET_DIR/src/Rocksmith2014.XML/Rocksmith2014.XML.csproj" -c Release --nologo >/dev/null

rm -rf "$DOTNET_APP_DIR"
mkdir -p "$DOTNET_APP_DIR"

cat > "$DOTNET_APP_DIR/NoteCheck.csproj" <<EOF
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="$DOTNET_DIR/src/Rocksmith2014.PSARC/Rocksmith2014.PSARC.fsproj" />
    <ProjectReference Include="$DOTNET_DIR/src/Rocksmith2014.XML/Rocksmith2014.XML.csproj" />
  </ItemGroup>
</Project>
EOF

cat > "$DOTNET_APP_DIR/Program.cs" <<'EOF'
using Rocksmith2014.PSARC;
using Rocksmith2014.XML;

static string Normalize(string text) =>
    new(text.Where(char.IsLetterOrDigit).Select(char.ToLowerInvariant).ToArray());

static bool IsTargetSong(string psarcPath, string entry, string title)
{
    var titleNorm = Normalize(title);
    var entryNorm = Normalize(entry);
    var fileNorm = Normalize(psarcPath);
    var hasInBetween = titleNorm.Contains("inbetween") || entryNorm.Contains("inbetween") || fileNorm.Contains("inbetween");
    var hasDream = titleNorm.Contains("dream") || entryNorm.Contains("dream") || fileNorm.Contains("dream");
    var hasDays = titleNorm.Contains("days") || entryNorm.Contains("days") || fileNorm.Contains("days");
    return hasInBetween && (hasDream || hasDays);
}

static ulong NoteChecksum(IEnumerable<Note> notes)
{
    ulong hash = 0xcbf29ce484222325;
    foreach (var row in notes
        .Select(note => $"{note.Time}:{note.String}:{note.Fret}:{note.Sustain}")
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

if (args.Length != 2)
{
    Console.Error.WriteLine("usage: NoteCheck <dlcDir> <summaryOut>");
    return 1;
}

var dlcDir = args[0];
var summaryOut = args[1];
var lines = new List<string>();
var printedTarget = false;

foreach (var psarcPath in Directory.EnumerateFiles(dlcDir, "*.psarc").OrderBy(x => x))
{
    using var psarc = PSARC.OpenFile(psarcPath);
    foreach (var entry in psarc.Manifest.Where(x => x.EndsWith(".xml", StringComparison.Ordinal)).OrderBy(x => x))
    {
        try
        {
            await using var entryStream = await psarc.GetEntryStream(entry);
            var tmpXml = Path.GetTempFileName();
            await using (var outFile = File.Create(tmpXml))
            {
                await entryStream.CopyToAsync(outFile);
            }

            var arr = InstrumentalArrangement.Load(tmpXml);
            File.Delete(tmpXml);

            foreach (var level in arr.Levels)
            {
                lines.Add(
                    $"{Path.GetFileName(psarcPath)}\t{entry}\t{level.Difficulty}\t{level.Notes.Count}\t{NoteChecksum(level.Notes):x16}"
                );
            }

            if (arr.Levels.Any(level => level.Notes.Count > 0) &&
                IsTargetSong(psarcPath, entry, arr.MetaData.Title ?? string.Empty))
            {
                printedTarget = true;
                Console.WriteLine($"\n[.NET] Target song: {arr.MetaData.Title ?? ""} ({psarcPath})");
                Console.WriteLine($"[.NET] Arrangement file: {entry}");
                foreach (var level in arr.Levels)
                {
                    Console.WriteLine($"[.NET] Difficulty {level.Difficulty} ({level.Notes.Count} notes)");
                    foreach (var note in level.Notes)
                        Console.WriteLine($"[.NET]   time={note.Time} string={note.String} fret={note.Fret} sustain={note.Sustain}");
                }
            }
        }
        catch
        {
            // Ignore non-instrumental XML files.
        }
    }
}

if (!printedTarget)
    throw new Exception("No DLC arrangement matched 'In between dreams' search in .NET parser.");

lines.Sort(StringComparer.Ordinal);
File.WriteAllLines(summaryOut, lines);
return 0;
EOF

dotnet run --project "$DOTNET_APP_DIR/NoteCheck.csproj" -- "$DLC_DIR" "$DOTNET_SUMMARY"

diff -u "$DOTNET_SUMMARY" "$RUST_SUMMARY"
echo "Rust and .NET note parsing summaries match."
