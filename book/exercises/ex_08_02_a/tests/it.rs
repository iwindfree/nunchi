use ex_08_02_a::collect_events;

#[test]
fn collects_all_paths() {
    let got = collect_events(vec![
        "a.rs".to_string(),
        "b.rs".to_string(),
        "c.rs".to_string(),
    ]);
    assert_eq!(got, vec!["a.rs", "b.rs", "c.rs"]);
}

#[test]
fn handles_empty() {
    assert!(collect_events(Vec::new()).is_empty());
}
