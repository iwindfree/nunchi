use ex_04_03_b::extensions;

#[test]
fn collects_extensions() {
    let paths = vec![
        "src/main.rs".to_string(),
        "Makefile".to_string(),
        "A.java".to_string(),
    ];
    assert_eq!(extensions(&paths), vec!["rs", "java"]);
}

#[test]
fn uses_filter_map() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(body.contains("filter_map"), "filter_map 을 쓰십시오");
    assert!(!body.contains("unwrap()"), "unwrap 이 남아 있습니다");
}
