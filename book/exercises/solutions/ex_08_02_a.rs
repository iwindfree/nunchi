// 정답
//
// 세 부분으로 나뉩니다.
//
//   let (tx, rx) = mpsc::channel();
//     tx 는 보내는 쪽, rx 는 받는 쪽입니다.
//
//   thread::spawn(move || { ... })
//     move 가 없으면 tx 와 paths 를 빌리려 하는데, 스레드가 언제까지
//     사는지 컴파일러가 알 수 없으므로 거부합니다. move 로 소유권을
//     통째로 넘겨야 합니다.
//
//   for p in rx { ... }
//     보내는 쪽이 전부 사라지면 반복이 자동으로 끝납니다.
//     스레드가 끝나면서 tx 가 사라지므로 따로 신호를 보낼 필요가 없습니다.
//
// nunchi 의 watch.rs 가 같은 구조입니다. 다만 거기서는 워처가 계속
// 살아 있어야 하므로 recv_timeout 으로 주기적으로 확인합니다.

use std::sync::mpsc;
use std::thread;

pub fn collect_events(paths: Vec<String>) -> Vec<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for p in paths {
            let _ = tx.send(p);
        }
    });

    rx.into_iter().collect()
}
