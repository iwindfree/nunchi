// derive 로 트레이트를 자동 구현하는 문제입니다.
//
// 아래 코드는 컴파일되지 않습니다. 테스트에서 다음을 하려고 합니다.
//   - assert_eq! 로 비교합니다        → PartialEq 가 필요합니다
//   - 실패 시 값을 출력합니다          → Debug 가 필요합니다
//   - HashSet 에 넣습니다             → Eq 와 Hash 가 필요합니다
//   - clone() 을 부릅니다             → Clone 이 필요합니다
//
// derive 목록에 필요한 것을 추가하십시오.

use std::collections::HashSet;

#[derive(Debug)]
pub struct NodeId(pub String);

pub fn unique_count(ids: &[NodeId]) -> usize {
    let set: HashSet<NodeId> = ids.iter().map(|id| id.clone()).collect();
    set.len()
}
