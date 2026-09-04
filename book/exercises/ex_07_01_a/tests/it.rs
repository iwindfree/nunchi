use ex_07_01_a::model::NodeId;
use ex_07_01_a::store::MemStore;

#[test]
fn modules_are_visible() {
    let mut s = MemStore::default();
    s.put(NodeId("file:a".to_string()));
    assert_eq!(s.count(), 1);
}
