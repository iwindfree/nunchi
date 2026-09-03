// 아래 함수를 완성하십시오.
//
// 언어 이름이 있으면 그대로 쓰고, 없으면 확장자에서 추정합니다.
// 추정은 비용이 드는 계산이라고 가정하고, 이름이 있을 때는 부르지
// 않아야 합니다.
//
// 힌트: unwrap_or 와 unwrap_or_else 중 어느 쪽입니까?(2.1장, 4.1장)

/// 호출 횟수를 세어 두는 전역입니다. 테스트가 이것을 확인합니다.
pub static GUESS_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn guess_from_extension(path: &str) -> String {
    GUESS_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    match path.rsplit_once('.') {
        Some((_, "rs")) => "rust".to_string(),
        Some((_, "java")) => "java".to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn language(declared: Option<String>, path: &str) -> String {
    todo!()
}
