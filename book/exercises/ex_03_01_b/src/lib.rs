// nunchi 의 이름 해소 로직을 단순하게 만든 것입니다.
//
// 후보 개수에 따라 네 갈래로 나뉩니다. match 와 가드를 써서 작성하십시오.
//
//   후보 0개  -> Resolution::None
//   후보 1개  -> Resolution::One(확신도 0.8)
//   후보 2~3개 -> Resolution::Many(확신도 0.8 / 개수)
//   후보 4개 이상 -> Resolution::TooMany
//
// 힌트: 가드는 `n if n <= 3 => ...` 형태입니다(3.1장).

#[derive(Debug, PartialEq)]
pub enum Resolution {
    None,
    One(f32),
    Many(f32),
    TooMany,
}

pub const MAX_CANDIDATES: usize = 3;

pub fn resolve(candidate_count: usize) -> Resolution {
    todo!()
}
