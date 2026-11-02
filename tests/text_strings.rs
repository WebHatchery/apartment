// Data integrity for the shipped player-facing text: every key the game
// looks up exists and uses the placeholders the game fills in.

use apartment::text::{parse, REQUIRED};

#[test]
fn shipped_text_strings_cover_every_key_with_its_placeholders() {
    let strings = parse(include_str!("../assets/text_strings.json"))
        .unwrap_or_else(|error| panic!("assets/text_strings.json: {error}"));
    for (key, _) in REQUIRED {
        assert!(strings.template(key).is_some(), "missing {key}");
    }
}

#[test]
fn a_template_missing_its_placeholder_is_rejected() {
    let broken = r#"{"events": {"rent_missed": "Someone missed rent"},
        "status": {}, "month": {}, "inspector": {}}"#;
    assert!(parse(broken).is_err());
}
