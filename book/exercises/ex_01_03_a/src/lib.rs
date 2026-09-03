// 아래 함수는 컴파일되지 않습니다.
//
// 목록의 첫 원소를 가리키는 참조를 만들어 두고, 그 상태에서 목록에 값을
// 추가하려고 합니다. 빌림 규칙 2에 걸립니다.
//
// 읽는 사람이 있는 동안에는 쓰는 사람이 있을 수 없습니다.
//
// 힌트: `first` 를 언제까지 살려 둘 필요가 있는지 생각해 보십시오.
//       빌림은 마지막으로 쓰이는 지점까지만 살아 있습니다(1.3장).

pub fn first_then_push(items: &mut Vec<String>) -> String {
    let first = &items[0];
    items.push("added".to_string()); // TODO: 이 줄에서 오류가 납니다
    first.clone()
}
