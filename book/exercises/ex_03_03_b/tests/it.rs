use ex_03_03_b::{is_packable, is_tag_boundary, NodeKind};

#[test]
fn checks_packable_kinds() {
    assert!(is_packable(NodeKind::Symbol));
    assert!(is_packable(NodeKind::Route));
    assert!(!is_packable(NodeKind::Commit));
}

#[test]
fn checks_tag_boundary() {
    assert!(is_tag_boundary(Some(' ')));
    assert!(is_tag_boundary(Some('>')));
    assert!(!is_tag_boundary(Some('K'))); // <selectKey> 를 걸러 냅니다
    assert!(!is_tag_boundary(None));
}

#[test]
fn uses_matches_macro() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("match "), "match 가 남아 있습니다");
    assert_eq!(body.matches("matches!").count(), 2, "matches! 를 두 번 쓰십시오");
}
