// 정답
//
// From 만 구현하면 Into 가 자동으로 생깁니다. 표준 라이브러리에
// "From<T> for U 가 있으면 Into<U> for T 도 있다" 는 규칙이
// 미리 정해져 있기 때문입니다.
//
// 그래서 Rust 에서는 항상 From 쪽을 구현합니다. Into 를 직접 구현하면
// 오히려 충돌이 납니다.
//
// 이 규칙이 ? 연산자와 연결됩니다(2.3장, 5.3장).
// ? 는 오류 타입이 다를 때 From 을 찾아 자동으로 변환합니다.
// anyhow 가 거의 모든 오류를 받아 주는 이유가 이것입니다.

#[derive(Debug, PartialEq)]
pub struct NodeId(pub String);

impl From<String> for NodeId {
    fn from(s: String) -> NodeId {
        NodeId(s)
    }
}
