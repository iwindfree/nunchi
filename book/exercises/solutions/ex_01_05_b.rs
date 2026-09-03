// 정답
//
// 세 함수 모두 &str 을 받게 바꿉니다. 그러면 describe 안의 clone 두 개도
// 함께 사라집니다. 빌린 것은 여러 번 넘겨도 되기 때문입니다(1.3장 규칙 1).
//
// 이것이 &str 을 쓰는 실질적 이득입니다. 서명 하나를 바꾸면 호출하는 쪽의
// to_string() 과 clone() 이 연쇄적으로 사라집니다.

pub fn is_test_path(path: &str) -> bool {
    path.contains("/test/") || path.ends_with("Test.java")
}

pub fn language_of(path: &str) -> Option<String> {
    let ext = path.rsplit_once('.')?.1;
    match ext {
        "rs" => Some("rust".to_string()),
        "java" => Some("java".to_string()),
        _ => None,
    }
}

pub fn describe(path: &str) -> String {
    let lang = language_of(path).unwrap_or_else(|| "unknown".to_string());
    let kind = if is_test_path(path) { "test" } else { "source" };
    format!("{} ({})", lang, kind)
}
