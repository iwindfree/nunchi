// 정답
//
// prefixed: name.to_string()
//   &str 을 String 으로 바꿉니다. 새 힙 공간을 잡고 문자를 복사합니다.
//
// ends_with: name.ends_with(&suffix)
//   ends_with 는 &str 을 받는데 suffix 가 String 이므로 & 를 붙여 빌려 줍니다.
//   Rust 가 &String 을 &str 로 자동 변환해 줍니다.
//
// 참고: ends_with 의 서명이 (String, String) 인 것 자체가 관례에 어긋납니다.
//       두 값 모두 보관하지 않고 읽기만 하므로 (&str, &str) 이 맞습니다.
//       이 문제에서는 서명을 바꾸지 말라고 했으므로 그대로 두었습니다.

pub fn prefixed(name: &str) -> String {
    name.to_string()
}

pub fn ends_with(name: String, suffix: String) -> bool {
    name.ends_with(&suffix)
}
