use ex_04_02_a::summarize;

#[test]
fn counts_and_measures() {
    let names = vec!["ab".to_string(), "cde".to_string()];
    assert_eq!(summarize(names), (2, 5));
}
