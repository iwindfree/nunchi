use ex_05_04_b::parse;

#[test]
fn reads_full_config() {
    let c = parse(
        r#"
languages = ["java"]
exclude = ["**/target/**"]
max_commits = 50
"#,
    )
    .unwrap();
    assert_eq!(c.languages, vec!["java"]);
    assert_eq!(c.max_commits, 50);
}

#[test]
fn fills_in_defaults() {
    let c = parse(r#"languages = ["rust"]"#).unwrap();
    assert_eq!(c.exclude, Vec::<String>::new());
    assert_eq!(c.max_commits, 1000);
}
