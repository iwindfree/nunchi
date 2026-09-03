use ex_01_04_b::{contains_edges, Edge, NodeId};

#[test]
fn makes_one_edge_per_symbol() {
    let file = NodeId("file:a.rs".to_string());
    let syms = vec![
        NodeId("sym:a.rs#foo".to_string()),
        NodeId("sym:a.rs#bar".to_string()),
    ];
    let edges = contains_edges(file, syms);
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].src, NodeId("file:a.rs".to_string()));
    assert_eq!(edges[1].src, NodeId("file:a.rs".to_string()));
    assert_eq!(edges[1].dst, NodeId("sym:a.rs#bar".to_string()));
}
