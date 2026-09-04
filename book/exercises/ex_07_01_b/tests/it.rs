use ex_07_01_b::{count_unique, group};

#[test]
fn counts_unique_names() {
    assert_eq!(count_unique(&["a", "b", "a"]), 2);
}

#[test]
fn groups_by_length() {
    let g = group(&["aa", "b", "cc"]);
    assert_eq!(g[&1], vec!["b"]);
    assert_eq!(g[&2].len(), 2);
}

// 이 문제는 동작이 아니라 코드 모양을 고치는 것이므로, 실제로 고쳤는지
// 소스를 읽어서 확인합니다.
#[test]
fn use_statements_replace_long_paths() {
    let src = include_str!("../src/lib.rs");

    assert!(
        src.contains("use std::collections::"),
        "파일 맨 위에 use std::collections::... 를 추가하십시오"
    );

    let long_paths = src.matches("std::collections::").count();
    assert!(
        long_paths <= 1,
        "본문에 std::collections:: 가 아직 {}번 남아 있습니다. \
         use 로 가져온 짧은 이름을 쓰십시오",
        long_paths - 1
    );
}
