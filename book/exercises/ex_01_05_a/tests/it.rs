use ex_01_05_a::{ends_with, prefixed};

#[test]
fn makes_owned_string() {
    assert_eq!(prefixed("OrderService"), "OrderService".to_string());
}

#[test]
fn checks_suffix() {
    assert!(ends_with("OrderServiceTest".to_string(), "Test".to_string()));
    assert!(!ends_with("OrderService".to_string(), "Test".to_string()));
}
