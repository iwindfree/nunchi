// 정답
//
//     declared.unwrap_or_else(|| guess_from_extension(path))
//
// unwrap_or 를 쓰면 안 되는 이유:
//   unwrap_or(guess_from_extension(path)) 라고 쓰면, 인자를 넘기기 위해
//   guess_from_extension 을 먼저 실행해야 합니다. declared 에 값이 있어도
//   실행됩니다. 테스트가 이것을 잡아냅니다.
//
// unwrap_or_else 는 클로저를 받습니다. 클로저는 "실행 방법" 이지 결과가
// 아니므로, 값이 없을 때만 실제로 실행됩니다.
//
// 같은 이유로 anyhow 의 with_context 도 클로저를 받습니다.
// 오류가 났을 때만 메시지를 만들기 위해서입니다(2.4장).

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
    declared.unwrap_or_else(|| guess_from_extension(path))
}
