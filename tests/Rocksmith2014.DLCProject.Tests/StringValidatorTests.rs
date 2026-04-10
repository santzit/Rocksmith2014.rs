//! Tests mirroring Rocksmith2014.DLCProject.Tests/StringValidatorTests.fs

use rocksmith2014_dlcproject::string_validator;

#[test]
fn non_alphanumeric_characters_are_removed_from_dlc_keys() {
    let result = string_validator::dlc_key("@ab1RÅ!魔?");
    assert_eq!(result, "ab1R");
}

#[test]
fn leading_non_alphanumeric_characters_are_removed_from_sort_fields() {
    let result = string_validator::sort_field("@!%ab1RÅ!魔?");
    assert_eq!(result, "ab1RÅ!魔?");
}

#[test]
fn characters_not_in_game_font_are_removed_from_fields() {
    let result = string_validator::field("Artist AΏԘ₯糨");
    assert_eq!(result, "Artist A");
}

#[test]
fn diacritics_are_removed_from_filenames() {
    let result = string_validator::file_name("Motörhead feat. Mötley Crüe");
    assert_eq!(result, "Motorhead-feat-Motley-Crue");
}

#[test]
fn invalid_characters_are_removed_from_filenames() {
    let result = string_validator::file_name(r#"A/"B"*C\D:E"#);
    assert_eq!(result, "ABCDE");
}

#[test]
fn ampersand_and_single_quotes_are_removed_from_filenames() {
    let result = string_validator::file_name("'A' & B");
    assert_eq!(result, "A-B");
}

#[test]
fn english_articles_are_removed_correctly() {
    let the = string_validator::remove_articles("The The");
    let a = string_validator::remove_articles("A Name");
    let an = string_validator::remove_articles("An Artist");

    assert_eq!(the, "The");
    assert_eq!(a, "Name");
    assert_eq!(an, "Artist");
}
