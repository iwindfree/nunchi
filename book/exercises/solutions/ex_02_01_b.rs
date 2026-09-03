// 정답
//
// display_name 에 unwrap_or_else 를 쓰는 이유:
//   "(anonymous)".to_string() 은 힙 할당을 합니다. unwrap_or 를 쓰면 이름이
//   있을 때도 이 문자열을 만들게 됩니다. unwrap_or_else 는 없을 때만
//   클로저를 실행하므로 낭비가 없습니다(2.1장).
//
// name_length 에는 unwrap_or(0) 을 씁니다.
//   0 을 만드는 데는 비용이 없으므로 클로저가 필요 없습니다.
//
// .map() 은 값이 있을 때만 함수를 적용하고 없으면 None 을 그대로 둡니다.
// 그래서 "없는 경우" 를 따로 처리하지 않아도 됩니다.

pub fn display_name(name: Option<String>) -> String {
    name.unwrap_or_else(|| "(anonymous)".to_string())
}

pub fn name_length(name: Option<String>) -> usize {
    name.map(|n| n.len()).unwrap_or(0)
}
