use ex_04_01_a::make_filter;

#[test]
fn filter_outlives_its_maker() {
    let f = make_filter("src/".to_string());
    assert!((f.check)("src/main.rs"));
    assert!(!(f.check)("docs/a.md"));
}
