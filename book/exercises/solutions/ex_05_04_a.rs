// 정답: #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//
// 각 트레이트가 무엇을 가능하게 하는지 정리하면 이렇습니다.
//
//   Debug      {:?} 로 출력합니다. assert_eq! 가 실패할 때 값을 보여 줍니다
//   Clone      .clone() 을 부를 수 있습니다
//   PartialEq  == 로 비교합니다. assert_eq! 에 필요합니다
//   Eq         PartialEq 보다 강한 약속입니다. HashSet 의 키가 되려면 필요합니다
//   Hash       해시값을 계산합니다. HashSet 과 HashMap 의 키에 필요합니다
//
// Eq 는 메서드가 없습니다. "이 타입은 자기 자신과 항상 같다" 는 표시일 뿐입니다.
// f32 는 NaN 때문에 이 성질이 없어서 Eq 가 아니며, 그래서 HashMap 의 키가
// 될 수 없습니다.
//
// nunchi 의 NodeId 에 정확히 이 조합이 붙어 있습니다.

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

pub fn unique_count(ids: &[NodeId]) -> usize {
    let set: HashSet<NodeId> = ids.iter().map(|id| id.clone()).collect();
    set.len()
}
