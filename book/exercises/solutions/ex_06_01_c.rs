// 정답
//
//     scored.into_iter().take(n).map(|(name, _)| name).collect()
//
// take 는 남은 것이 부족해도 멈추지 않고 있는 만큼만 줍니다.
// 그래서 길이를 따로 확인할 필요가 없습니다.
//
// 슬라이스로 자르는 방법도 있습니다.
//
//     let end = n.min(scored.len());
//     scored[..end].iter().map(|(name, _)| name.clone()).collect()
//
// 다만 이쪽은 clone 이 필요합니다. 슬라이스는 빌린 것이라 원소를
// 가져올 수 없기 때문입니다. into_iter 는 Vec 을 소비하면서 원소의
// 소유권을 주므로 clone 이 필요 없습니다.
//
// nunchi 의 pack.rs 에서도 후보를 자를 때 truncate 와 take 를 씁니다.

pub fn top_n(mut scored: Vec<(String, f32)>, n: usize) -> Vec<String> {
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.into_iter().take(n).map(|(name, _)| name).collect()
}
