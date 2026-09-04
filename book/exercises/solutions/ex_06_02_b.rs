// 정답
//
//     *counts.entry(name.to_string()).or_insert(0) += 1;
//
// 한 줄을 풀어 읽으면 이렇습니다.
//   entry(...)      그 키의 자리를 얻습니다
//   .or_insert(0)   없으면 0 을 넣고, &mut usize 를 돌려줍니다
//   *... += 1       참조가 가리키는 곳의 값을 1 늘립니다
//
// 앞에 붙은 * 가 참조 해제(dereference)입니다. or_insert 가 돌려주는 것은
// 값이 아니라 "값을 가리키는 참조" 이므로, 그 자리의 값을 바꾸려면
// * 로 따라 들어가야 합니다.
//
// nunchi 의 resolve.rs 에 같은 코드가 있습니다.
//
//     *self.0.entry(name.to_string()).or_default() += 1;

use std::collections::HashMap;

pub fn tally(names: &[&str]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for name in names {
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    counts
}
