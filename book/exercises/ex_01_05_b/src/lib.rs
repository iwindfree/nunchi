// 아래 코드는 컴파일됩니다. 다만 불필요한 힙 할당이 일어납니다.
//
// 세 함수 모두 값을 보관하지 않고 읽기만 합니다. 그런데 String 을 받으므로
// 호출하는 쪽에서 매번 .to_string() 을 불러야 합니다.
//
// 세 함수의 서명을 &str 을 받도록 바꾸고, 테스트가 통과하게 만드십시오.
// (테스트 파일도 함께 보십시오. 호출 방식이 바뀝니다.)

pub fn is_test_path(path: String) -> bool {
    path.contains("/test/") || path.ends_with("Test.java")
}

pub fn language_of(path: String) -> Option<String> {
    let ext = path.rsplit_once('.')?.1;
    match ext {
        "rs" => Some("rust".to_string()),
        "java" => Some("java".to_string()),
        _ => None,
    }
}

pub fn describe(path: String) -> String {
    let lang = language_of(path.clone()).unwrap_or_else(|| "unknown".to_string());
    let kind = if is_test_path(path.clone()) { "test" } else { "source" };
    format!("{} ({})", lang, kind)
}
