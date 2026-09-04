use ex_05_02_a::{describe, MemStore, NodeStore};

#[test]
fn stores_and_counts() {
    let mut s = MemStore::default();
    assert_eq!(s.count(), 0);
    s.put("file:a".to_string());
    s.put("file:b".to_string());
    assert_eq!(s.count(), 2);
}

#[test]
fn describe_accepts_any_store() {
    let mut s = MemStore::default();
    s.put("file:a".to_string());
    assert_eq!(describe(&s), "1 nodes");
}
