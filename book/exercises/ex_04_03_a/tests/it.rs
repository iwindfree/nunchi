use ex_04_03_a::parse_depth;

#[test]
fn parses_valid_numbers() {
    assert_eq!(parse_depth(Some(" 3 ".into())), Some(3));
    assert_eq!(parse_depth(Some("1000".into())), Some(1000));
}

#[test]
fn returns_none_for_invalid_or_missing() {
    assert_eq!(parse_depth(Some("deep".into())), None);
    assert_eq!(parse_depth(None), None);
}
