use ex_01_06_a::{deeper, strip_node_prefix};

#[test]
fn picks_deeper_path() {
    assert_eq!(deeper("a/b/c.rs", "x.rs"), "a/b/c.rs");
    assert_eq!(deeper("x.rs", "a/b/c.rs"), "a/b/c.rs");
}

#[test]
fn strips_prefix() {
    assert_eq!(strip_node_prefix("file:api/a.rs"), "api/a.rs");
    assert_eq!(strip_node_prefix("plain"), "plain");
}
