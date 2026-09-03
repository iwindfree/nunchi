// 아래 코드는 컴파일되지 않습니다.
//
// move 를 붙였는데 그럴 필요가 없었습니다. 클로저는 이 함수 안에서만
// 쓰이고 밖으로 나가지 않습니다. 그런데 move 때문에 소유권을 가져가서
// 아래에서 excludes 를 쓸 수 없게 됐습니다.
//
// 고치십시오.

pub fn count_excluded(paths: &[String], excludes: Vec<String>) -> (usize, usize) {
    let is_excluded = move |p: &String| excludes.iter().any(|e| p.contains(e));
    let excluded = paths.iter().filter(|p| is_excluded(p)).count();
    (excluded, excludes.len()) // TODO: excludes 를 쓸 수 없습니다
}
