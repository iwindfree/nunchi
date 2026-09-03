use ex_01_03_b::summarize;

#[test]
fn summarizes_names() {
    let names = vec!["ab".to_string(), "cde".to_string()];
    assert_eq!(summarize(names), "2 names, 5 chars");
}
