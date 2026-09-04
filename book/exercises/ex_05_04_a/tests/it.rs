use ex_05_04_a::{unique_count, NodeId};

#[test]
fn counts_unique_ids() {
    let ids = vec![
        NodeId("a".to_string()),
        NodeId("b".to_string()),
        NodeId("a".to_string()),
    ];
    assert_eq!(unique_count(&ids), 2);
}
