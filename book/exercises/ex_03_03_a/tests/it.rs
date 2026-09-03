use ex_03_03_a::{location, Node};

#[test]
fn builds_location() {
    let n = Node {
        path: Some("src/A.java".into()),
        span: Some((10, 25)),
        lang: Some("java".into()),
    };
    assert_eq!(location(&n).as_deref(), Some("[java] src/A.java:10-25"));
}

#[test]
fn returns_none_when_any_field_missing() {
    let n = Node { path: None, span: Some((1, 2)), lang: Some("rs".into()) };
    assert_eq!(location(&n), None);
    let n = Node { path: Some("a".into()), span: None, lang: Some("rs".into()) };
    assert_eq!(location(&n), None);
    let n = Node { path: Some("a".into()), span: Some((1, 2)), lang: None };
    assert_eq!(location(&n), None);
}

#[test]
fn is_flat() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("if let"), "if let 이 남아 있습니다");
    assert_eq!(
        body.matches("else {").count(),
        3,
        "let ... else 를 세 번 쓰십시오"
    );
}
