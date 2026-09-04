// Arc 로 데이터를 공유하는 문제입니다.
//
// 아래 코드는 컴파일되지 않습니다. 같은 설정을 여러 작업이 함께
// 읽어야 하는데, 소유권은 하나뿐이라 나눠 줄 수 없습니다.
//
// Arc 로 감싸서 고치십시오.
//
// Arc 는 "여러 주인이 함께 소유하는" 장치입니다. 복사하면 데이터가
// 아니라 참조 횟수만 늘어나고, 마지막 주인이 사라질 때 데이터가
// 정리됩니다.
//
// 힌트:
//   let shared = Arc::new(config);
//   let a = Arc::clone(&shared);   데이터가 아니라 횟수만 복사됩니다

use std::sync::Arc;

pub struct Config {
    pub name: String,
    pub budget: usize,
}

pub async fn run_two(config: Config) -> (String, String) {
    // TODO: Arc 를 써서 두 작업이 같은 설정을 읽게 만드십시오
    let first = tokio::spawn(async move { format!("{}:{}", config.name, config.budget) });
    let second = tokio::spawn(async move { format!("{}!", config.name) });

    (first.await.unwrap(), second.await.unwrap())
}
