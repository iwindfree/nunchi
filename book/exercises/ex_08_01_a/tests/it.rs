use ex_08_01_a::EdgeKind;

#[test]
fn converts_to_str() {
    assert_eq!(EdgeKind::Calls.as_str(), "calls");
    assert_eq!(EdgeKind::Imports.as_str(), "imports");
}

#[test]
fn parses_from_str() {
    assert_eq!(EdgeKind::parse("injects"), Some(EdgeKind::Injects));
    assert_eq!(EdgeKind::parse("nope"), None);
}
