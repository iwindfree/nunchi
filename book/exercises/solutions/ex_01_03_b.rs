// 정답: total_length 가 &[String] 을 받게 바꾸고, 호출할 때 &names 로 넘깁니다.
//
// &[String] 은 "빌린 문자열 목록" 입니다. Vec<String> 을 넘길 때 &names 라고
// 쓰면 Rust 가 자동으로 슬라이스로 바꿔 줍니다.
//
// 왜 &Vec<String> 이 아니라 &[String] 인가:
//   &[String] 이 더 넓게 받습니다. Vec 뿐 아니라 배열과 다른 슬라이스도
//   넘길 수 있습니다. 관례상 &Vec<T> 보다 &[T] 를 씁니다(6.1장).

pub fn total_length(names: &[String]) -> usize {
    names.iter().map(|n| n.len()).sum()
}

pub fn summarize(names: Vec<String>) -> String {
    let total = total_length(&names);
    format!("{} names, {} chars", names.len(), total)
}
