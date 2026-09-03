use ex_03_02_b::drain_order;
use std::collections::VecDeque;

#[test]
fn drains_in_order() {
    let q: VecDeque<u32> = vec![1, 2, 3].into();
    assert_eq!(drain_order(q), vec![1, 2, 3]);
}

#[test]
fn handles_empty_queue() {
    assert_eq!(drain_order(VecDeque::new()), Vec::<u32>::new());
}

#[test]
fn uses_while_let() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(body.contains("while let"), "while let 을 쓰십시오");
}
