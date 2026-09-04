use ex_05_02_b::NodeId;

#[test]
fn node_id_is_displayable() {
    let id = NodeId("file:api/A.java".to_string());
    assert_eq!(format!("{}", id), "file:api/A.java");
    assert_eq!(id.to_string(), "file:api/A.java");
}
