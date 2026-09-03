use ex_02_01_a::{extension, language_of};

#[test]
fn extension_may_be_missing() {
    assert_eq!(extension("main.rs").as_deref(), Some("rs"));
    assert_eq!(extension("Makefile"), None);
}

#[test]
fn language_may_be_unknown() {
    assert_eq!(language_of("a.rs"), Some("rust"));
    assert_eq!(language_of("a.txt"), None);
    assert_eq!(language_of("Makefile"), None);
}
