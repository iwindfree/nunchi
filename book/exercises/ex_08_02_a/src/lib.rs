// 채널로 값을 모으는 문제입니다.
//
// nunchi 의 파일 워처는 다른 스레드에서 오는 변경 알림을 채널로 받습니다.
// 여기서는 그 구조를 축소했습니다.
//
// `collect_events` 를 완성하십시오.
//   1. 채널을 만듭니다 (mpsc::channel())
//   2. 보내는 쪽을 다른 스레드로 옮겨 paths 를 하나씩 보냅니다
//   3. 받는 쪽에서 전부 모아 Vec 으로 돌려줍니다
//
// 힌트:
//   보내는 쪽을 스레드로 옮기려면 move 클로저가 필요합니다(4.1장).
//   스레드가 끝나면 보내는 쪽이 사라지고, 그때 받는 쪽 반복이 끝납니다.
//   그래서 for 로 받으면 자연히 멈춥니다.

use std::sync::mpsc;
use std::thread;

pub fn collect_events(paths: Vec<String>) -> Vec<String> {
    // TODO: 여기를 완성하십시오
    todo!()
}
