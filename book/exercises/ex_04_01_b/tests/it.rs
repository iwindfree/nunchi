use ex_04_01_b::count_excluded;

#[test]
fn counts_excluded_paths() {
    let paths = vec![
        "src/main.rs".to_string(),
        "target/debug/x".to_string(),
        "node_modules/a.js".to_string(),
    ];
    let excludes = vec!["target".to_string(), "node_modules".to_string()];
    assert_eq!(count_excluded(&paths, excludes), (2, 2));
}
