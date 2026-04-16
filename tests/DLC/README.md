Rocksmith 2014 DLC / Custom DLC for testing propose only.

Use `tests/check_song_notes_against_dotnet.sh` to:
- clone `Rocksmith2014.NET` into `tests/rocksmith2014-net` (tag `v3.5.0`)
- compare Rust vs .NET note parsing summaries for all `.psarc` files in this folder
- print the notes for the "In between dreams" target song match
