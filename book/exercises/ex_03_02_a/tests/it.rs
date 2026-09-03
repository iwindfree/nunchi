use ex_03_02_a::{tally, Node, Stats};

#[test]
fn counts_present_fields() {
    let mut stats = Stats::default();
    tally(&Node { span: Some((1, 5)), doc: None }, &mut stats);
    tally(&Node { span: None, doc: Some("d".into()) }, &mut stats);
    tally(&Node { span: Some((2, 3)), doc: Some("e".into()) }, &mut stats);
    assert_eq!(stats.with_span, 2);
    assert_eq!(stats.with_doc, 2);
}

#[test]
fn uses_if_let() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("match "), "match 가 남아 있습니다");
    assert_eq!(body.matches("if let").count(), 2, "if let 을 두 번 쓰십시오");
}
