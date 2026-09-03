use ex_01_04_a::build_report;

#[test]
fn builds_report() {
    let out = build_report(
        "api".to_string(),
        vec!["a.java".to_string(), "b.java".to_string()],
    );
    assert_eq!(out, "repo: api\nfiles: 2\nfirst: a.java");
}

/// 이 문제는 "고치기" 가 아니라 "개선하기" 입니다.
/// 처음부터 동작은 하므로, 불필요한 복사가 사라졌는지를 직접 검사합니다.
#[test]
fn unnecessary_clones_are_removed() {
    let source = include_str!("../src/lib.rs");
    // 주석은 빼고 셉니다. 설명에 적어 둔 글자까지 세면 안 됩니다.
    // .cloned() 는 남아 있어야 합니다. Option<&String> 을 String 으로
    // 바꾸는 정당한 복사이기 때문입니다.
    let count = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| line.matches(".clone()").count())
        .sum::<usize>();
    assert_eq!(
        count, 0,
        "불필요한 .clone() 이 {count}개 남아 있습니다. \
         값을 읽기만 하는 자리에는 복사가 필요 없습니다."
    );
}
