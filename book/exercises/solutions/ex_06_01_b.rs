// 정답: retain 을 씁니다.
//
//     paths.retain(|p| !p.contains("/test/"));
//
// retain 은 "조건이 참인 것만 남긴다" 는 뜻이므로 조건을 뒤집어야 합니다.
//
// 원래 코드가 왜 안 되는지가 중요합니다.
//   iter() 가 paths 를 빌리고 있는 동안 remove 가 paths 를 바꾸려 합니다.
//   읽는 사람이 있는데 쓰는 사람이 끼어드는 상황이므로 규칙 2 위반입니다.
//
// 설령 컴파일된다 해도 논리가 틀립니다. 원소를 지우면 뒤 원소들의 인덱스가
// 하나씩 당겨지므로 건너뛰는 원소가 생깁니다. 다른 언어에서 흔히 나는
// 버그인데, Rust 는 그런 코드를 아예 못 쓰게 막습니다.

pub fn drop_tests(paths: &mut Vec<String>) {
    paths.retain(|p| !p.contains("/test/"));
}
