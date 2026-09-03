// 정답: move 를 지웁니다.
//
//     let is_excluded = |p: &String| excludes.iter().any(|e| p.contains(e));
//
// 클로저가 excludes 를 읽기만 하므로 컴파일러가 빌림을 고릅니다.
// 그러면 아래에서 excludes.len() 을 부를 수 있습니다.
//
// move 는 필요할 때만 붙입니다. 클로저가 만들어진 곳보다 오래 살아야 할
// 때가 그런 경우이며, 그렇지 않으면 오히려 방해가 됩니다.
//
// 판단 기준:
//   클로저가 함수 밖으로 나가는가? (반환값, 구조체에 저장, 다른 스레드)
//   나가면 move, 안 나가면 붙이지 않습니다.

pub fn count_excluded(paths: &[String], excludes: Vec<String>) -> (usize, usize) {
    let is_excluded = |p: &String| excludes.iter().any(|e| p.contains(e));
    let excluded = paths.iter().filter(|p| is_excluded(p)).count();
    (excluded, excludes.len())
}
