// 정답
//
//     pub fn deeper<'a>(a: &'a str, b: &'a str) -> &'a str
//
// 읽는 법: "a 와 b 와 반환값이 모두 같은 수명을 갖는다."
// 실제 의미는 "반환값은 a 와 b 중 짧게 사는 쪽만큼만 살아 있다" 입니다.
//
// strip_node_prefix 에 표기가 필요 없는 이유:
//   참조를 하나만 받으므로 컴파일러가 "반환값은 id 에서 왔다" 고 추론합니다.
//   이것을 수명 생략 규칙이라고 부르며, 실제 코드의 대부분이 여기에 해당합니다.
//
// nunchi 에 수명 표기가 여섯 번뿐인 이유도 같습니다. 대부분의 함수가
// 참조를 하나만 받거나, 아예 소유한 값을 돌려줍니다.

pub fn deeper<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.matches('/').count() >= b.matches('/').count() { a } else { b }
}

pub fn strip_node_prefix(id: &str) -> &str {
    id.split_once(':').map(|(_, rest)| rest).unwrap_or(id)
}
