use ex_01_03_a::first_then_push;

#[test]
fn returns_first_and_appends() {
    let mut items = vec!["a".to_string(), "b".to_string()];
    assert_eq!(first_then_push(&mut items), "a");
    assert_eq!(items, vec!["a", "b", "added"]);
}
