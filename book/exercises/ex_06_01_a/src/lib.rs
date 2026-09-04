// 아래 함수는 컴파일되지만 쓰기 불편합니다.
//
// Vec<String> 만 받으므로 배열이나 슬라이스를 넘길 수 없습니다.
// 서명을 바꿔서 더 넓게 받게 만드십시오.
//
// 힌트: 목록을 읽기만 하는 함수는 &Vec<T> 가 아니라 &[T] 를 받습니다(6.1장).

pub fn longest(names: &Vec<String>) -> Option<&String> {
    names.iter().max_by_key(|n| n.len())
}
