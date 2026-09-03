// 정답
//
// pop_front() 가 Some(값) 을 돌려주는 동안 반복하고, 비어서 None 이
// 나오면 반복이 끝납니다.
//
// 이것이 nunchi 의 너비 우선 탐색과 같은 구조입니다.
// store/sqlite.rs 의 paths() 가 큐에 경로를 넣어 두고 이 방식으로 비웁니다.

use std::collections::VecDeque;

pub fn drain_order(mut queue: VecDeque<u32>) -> Vec<u32> {
    let mut visited = Vec::new();
    while let Some(item) = queue.pop_front() {
        visited.push(item);
    }
    visited
}
