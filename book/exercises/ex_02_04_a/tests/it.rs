use ex_02_04_a::read_number;

#[test]
fn reads_valid_number() {
    let n = read_number("nunchi.toml", "max_commits = 1000", "max_commits").unwrap();
    assert_eq!(n, 1000);
}

#[test]
fn error_mentions_file_and_key() {
    let err = read_number("nunchi.toml", "max_commits = deep", "max_commits")
        .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("nunchi.toml"), "파일 이름이 없습니다: {msg}");
    assert!(msg.contains("max_commits"), "설정 이름이 없습니다: {msg}");
    // 원래 오류도 남아 있어야 합니다
    assert!(msg.contains("invalid digit"), "근본 원인이 사라졌습니다: {msg}");
}

#[test]
fn missing_equals_also_has_context() {
    let err = read_number("nunchi.toml", "max_commits", "max_commits").unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("nunchi.toml"), "파일 이름이 없습니다: {msg}");
}
