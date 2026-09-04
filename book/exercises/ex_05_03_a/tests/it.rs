use ex_05_03_a::NodeId;

#[test]
fn converts_with_from() {
    let id = NodeId::from("file:a".to_string());
    assert_eq!(id, NodeId("file:a".to_string()));
}

#[test]
fn into_comes_for_free() {
    let id: NodeId = "file:b".to_string().into();
    assert_eq!(id, NodeId("file:b".to_string()));
}
