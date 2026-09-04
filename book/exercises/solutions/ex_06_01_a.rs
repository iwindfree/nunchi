// 정답: `&Vec<String>` 을 `&[String]` 으로 바꿉니다.
//
// &[T] 는 슬라이스이며 "연속된 T 여러 개를 빌린 것" 을 뜻합니다.
// Vec, 배열, 다른 슬라이스가 모두 &[T] 로 자동 변환됩니다.
//
// 반대로 &Vec<T> 는 Vec 만 받습니다. 배열은 받지 못합니다.
//
// 규칙:
//   목록을 읽기만 하면 &[T]
//   목록에 넣거나 빼야 하면 &mut Vec<T>
//
// nunchi 의 함수들이 &[Node], &[Edge], &[String] 을 받는 이유가 이것입니다.
//
//     pub fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize>

pub fn longest(names: &[String]) -> Option<&String> {
    names.iter().max_by_key(|n| n.len())
}
