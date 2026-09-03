use ex_02_01_b::{display_name, name_length};

#[test]
fn falls_back_when_missing() {
    assert_eq!(display_name(Some("OrderService".into())), "OrderService");
    assert_eq!(display_name(None), "(anonymous)");
}

#[test]
fn measures_length() {
    assert_eq!(name_length(Some("abc".into())), 3);
    assert_eq!(name_length(None), 0);
}
