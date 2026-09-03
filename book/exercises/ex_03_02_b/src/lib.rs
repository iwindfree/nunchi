// nunchi 의 그래프 순회를 단순하게 만든 것입니다.
//
// 큐가 빌 때까지 꺼내면서 방문한 노드를 순서대로 모으십시오.
// while let 을 쓰십시오(3.2장).
//
// 힌트: VecDeque 의 pop_front() 는 비면 None 을 돌려줍니다.

use std::collections::VecDeque;

pub fn drain_order(mut queue: VecDeque<u32>) -> Vec<u32> {
    let mut visited = Vec::new();
    // TODO: while let 으로 큐를 비우면서 visited 에 넣으십시오
    visited
}
