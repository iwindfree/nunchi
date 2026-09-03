use ex_01_01_b::build;

#[test]
fn builds_id_and_edge() {
    let (id, edge) = build("src/main.rs".to_string());
    assert_eq!(id, "file:src/main.rs");
    assert_eq!(edge, "contains:src/main.rs");
}
