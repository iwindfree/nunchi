// 정답: 로그 문자열을 먼저 만들고 나중에 push 합니다.
//
// `format!` 은 `name` 을 빌려서 읽기만 합니다(1.3장). 그래서 그 뒤에도
// `name` 의 소유권이 남아 있고 push 로 넘길 수 있습니다.
//
// 순서를 바꾸는 것만으로 clone 을 피할 수 있는 경우가 실제로 많습니다.
// 소유권을 넘기는 동작을 마지막에 두는 것이 요령입니다.

pub fn record(name: String, names: &mut Vec<String>) -> String {
    let msg = format!("added {}", name);
    names.push(name);
    msg
}
