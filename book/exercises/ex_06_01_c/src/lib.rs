// 슬라이스를 다루는 문제입니다.
//
// `top_n` 은 점수가 높은 순으로 정렬한 뒤 앞에서 n 개를 돌려줍니다.
// 다만 목록이 n 개보다 짧으면 있는 만큼만 돌려주어야 합니다.
//
// 힌트: 슬라이스를 자를 때 범위를 넘으면 프로그램이 멈춥니다(panic).
//       n.min(길이) 를 쓰거나 다른 방법을 찾으십시오.

pub fn top_n(mut scored: Vec<(String, f32)>, n: usize) -> Vec<String> {
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    // TODO: 여기를 완성하십시오
    todo!()
}
