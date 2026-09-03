use ex_04_02_c::{total_code_lines, FileInfo};

#[test]
fn sums_only_code_files() {
    let files = vec![
        FileInfo { path: "a.rs".into(), lines: 100, is_code: true },
        FileInfo { path: "README.md".into(), lines: 50, is_code: false },
        FileInfo { path: "b.java".into(), lines: 30, is_code: true },
    ];
    assert_eq!(total_code_lines(&files), 130);
}

#[test]
fn handles_empty() {
    assert_eq!(total_code_lines(&[]), 0);
}

#[test]
fn no_mutable_state() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("let mut"), "mut 변수가 남아 있습니다");
    assert!(!body.contains("for "), "반복문이 남아 있습니다");
}
