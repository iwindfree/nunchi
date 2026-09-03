use ex_01_01_a::record;

#[test]
fn records_and_reports() {
    let mut names = Vec::new();
    let msg = record("OrderService".to_string(), &mut names);
    assert_eq!(msg, "added OrderService");
    assert_eq!(names, vec!["OrderService"]);
}
