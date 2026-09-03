// 정답
//
// 가드(`n if ...`)는 값을 이름에 받은 다음 조건을 겁니다.
// 위에서 아래로 순서대로 확인하므로, 0 과 1 을 먼저 걸러 낸 뒤
// 가드가 2 와 3 을 잡고, 나머지가 `_` 로 갑니다.
//
// 후보가 넷 이상이면 포기하는 이유:
//   get 이나 build 같은 흔한 이름은 코드베이스 전체에 수십 개 있습니다.
//   그것을 모두 연결하면 그래프가 잡음으로 뒤덮입니다.
//
// 후보가 하나여도 확신도가 1.0 이 아닌 이유:
//   이름이 같다는 사실은 타입을 해소한 것과 다릅니다. 어디까지나 추정입니다.

#[derive(Debug, PartialEq)]
pub enum Resolution {
    None,
    One(f32),
    Many(f32),
    TooMany,
}

pub const MAX_CANDIDATES: usize = 3;

pub fn resolve(candidate_count: usize) -> Resolution {
    match candidate_count {
        0 => Resolution::None,
        1 => Resolution::One(0.8),
        n if n <= MAX_CANDIDATES => Resolution::Many(0.8 / n as f32),
        _ => Resolution::TooMany,
    }
}
