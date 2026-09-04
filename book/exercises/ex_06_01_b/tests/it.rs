use ex_06_01_b::drop_tests;

#[test]
fn removes_test_paths() {
    let mut paths = vec![
        "src/main.rs".to_string(),
        "src/test/a.rs".to_string(),
        "src/lib.rs".to_string(),
        "x/test/b.rs".to_string(),
    ];
    drop_tests(&mut paths);
    assert_eq!(paths, vec!["src/main.rs", "src/lib.rs"]);
}
