use ex_03_01_a::{is_structural, EdgeKind};

#[test]
fn classifies_all_kinds() {
    assert!(is_structural(EdgeKind::Calls));
    assert!(is_structural(EdgeKind::Injects));
    assert!(is_structural(EdgeKind::CallsApi));
    assert!(!is_structural(EdgeKind::ModifiedBy));
    assert!(!is_structural(EdgeKind::AuthoredBy));
}

/// `_` 로 묶지 않았는지 확인합니다.
#[test]
fn no_wildcard_arm() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !body.contains("_ =>"),
        "`_` 로 묶으면 열거형에 값을 추가해도 컴파일러가 알려 주지 않습니다"
    );
}
