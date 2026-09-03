// 아래 함수 두 개는 컴파일되지 않습니다.
//
// String 과 &str 을 잘못 쓰고 있습니다. 타입이 맞도록 고치십시오.
// 함수 서명은 바꾸지 말고 본문만 고치십시오.
//
// 힌트: &str 에서 String 을 만드는 방법은 .to_string() 입니다.
//       String 에서 &str 을 얻는 방법은 & 를 붙이는 것입니다(1.5장).

/// 접두를 붙여 새 식별자를 만듭니다.
pub fn prefixed(name: &str) -> String {
    name // TODO: 반환 타입은 String 인데 &str 을 돌려주고 있습니다
}

/// 이름이 접미로 끝나는지 봅니다.
pub fn ends_with(name: String, suffix: String) -> bool {
    name.ends_with(suffix) // TODO: ends_with 는 &str 을 받습니다
}
