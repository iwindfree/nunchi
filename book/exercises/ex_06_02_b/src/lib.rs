// 아래 함수는 컴파일되지 않습니다.
//
// 미해소 호출 이름의 등장 횟수를 세려고 합니다.
// HashMap 에서 값을 꺼내 1 을 더하려는데 빌림 규칙에 걸립니다.
//
// 힌트: entry(...).or_insert(0) 이 &mut usize 를 돌려줍니다.
//       거기에 *를 붙여 값을 바꿉니다(1.3장의 참조 해제).

use std::collections::HashMap;

pub fn tally(names: &[&str]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for name in names {
        // TODO: 이 부분을 고치십시오
        let current = counts.get(*name);
        counts.insert(name.to_string(), current + 1);
    }
    counts
}
