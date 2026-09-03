use ex_00_03_a::{extension, line_count};

#[test]
fn extension_is_optional() {
    assert_eq!(extension("main.rs").as_deref(), Some("rs"));
    assert_eq!(extension("Makefile"), None);
}

#[test]
fn counts_lines() {
    assert_eq!(line_count("a\nb\nc"), 3);
    assert_eq!(line_count(""), 0);
}
