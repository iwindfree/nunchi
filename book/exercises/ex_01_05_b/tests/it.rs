use ex_01_05_b::{describe, is_test_path, language_of};

#[test]
fn accepts_string_literals_directly() {
    // 서명이 &str 이면 아래처럼 .to_string() 없이 부를 수 있어야 합니다.
    assert!(is_test_path("src/test/A.java"));
    assert!(!is_test_path("src/main/A.java"));
    assert_eq!(language_of("a.rs").as_deref(), Some("rust"));
    assert_eq!(language_of("Makefile"), None);
}

#[test]
fn describes_path() {
    assert_eq!(describe("src/test/A.java"), "java (test)");
    assert_eq!(describe("src/main/A.rs"), "rust (source)");
}
