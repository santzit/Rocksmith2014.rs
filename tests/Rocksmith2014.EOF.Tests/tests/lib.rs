use rocksmith2014_eof::EofProTracks;

#[test]
fn test_eof_pro_tracks_default() {
    let tracks = EofProTracks::default();
    assert!(tracks.part_guitar.is_empty());
    assert!(tracks.part_bass.is_empty());
    assert!(tracks.part_bonus.is_none());
    assert!(tracks.part_vocals.is_none());
}
