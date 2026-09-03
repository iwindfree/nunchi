use ex_02_03_b::extension_lower;

#[test]
fn lowercases_extension() {
    assert_eq!(extension_lower("src/Main.JAVA").as_deref(), Some("java"));
    assert_eq!(extension_lower("a/b/c.rs").as_deref(), Some("rs"));
}

#[test]
fn returns_none_without_extension() {
    assert_eq!(extension_lower("Makefile"), None);
    assert_eq!(extension_lower("src/Makefile"), None);
}
