// 아래 함수는 컴파일되지 않습니다.
//
// 노드 이름을 목록에 넣고, 그 이름을 다시 써서 로그 문자열을 만들려고 합니다.
// 목록에 넣는 순간 소유권이 넘어가서 이름을 더 쓸 수 없습니다.
//
// `.clone()` 을 쓰지 않고 고치십시오.
// 힌트: 순서를 바꾸면 됩니다. 무엇을 먼저 해야 할까요?

pub fn record(name: String, names: &mut Vec<String>) -> String {
    names.push(name);
    format!("added {}", name) // TODO: 이 줄에서 오류가 납니다
}
