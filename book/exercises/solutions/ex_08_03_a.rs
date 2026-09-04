// 정답
//
// Arc::new 로 감싸고, 작업마다 Arc::clone 으로 나눠 줍니다.
//
// Arc::clone 은 데이터를 복사하지 않습니다. 참조 횟수를 하나 늘리고
// 같은 데이터를 가리키는 새 손잡이를 돌려줄 뿐입니다. 비용이 거의 없습니다.
//
// 이름이 Atomically Reference Counted 의 줄임말입니다.
//   Reference Counted   몇 명이 쓰고 있는지 셉니다
//   Atomically          여러 스레드가 동시에 세도 안전합니다
//
// 한 스레드 안에서만 쓴다면 Rc 가 더 빠릅니다. 다만 nunchi 의 MCP 서버는
// tokio 가 작업을 여러 스레드에 나눠 돌리므로 Arc 가 필요합니다.
//
// nunchi 에서 Arc 는 두 곳에만 나옵니다. MCP 도구 스키마를 여러 요청이
// 공유하는 자리입니다.

use std::sync::Arc;

pub struct Config {
    pub name: String,
    pub budget: usize,
}

pub async fn run_two(config: Config) -> (String, String) {
    let shared = Arc::new(config);

    let a = Arc::clone(&shared);
    let first = tokio::spawn(async move { format!("{}:{}", a.name, a.budget) });

    let b = Arc::clone(&shared);
    let second = tokio::spawn(async move { format!("{}!", b.name) });

    (first.await.unwrap(), second.await.unwrap())
}
