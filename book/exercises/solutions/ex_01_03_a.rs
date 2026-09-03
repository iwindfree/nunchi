// 정답: 첫 원소를 먼저 복사해서 빌림을 끝낸 다음 push 합니다.
//
//     let first = items[0].clone();
//     items.push("added".to_string());
//     first
//
// 이렇게 하면 `items[0]` 에서 빌린 것이 그 줄에서 바로 끝납니다.
// 다음 줄에서는 빌린 것이 없으므로 &mut 를 얻을 수 있습니다.
//
// 원래 코드가 안 되는 이유:
//   `first` 가 세 번째 줄까지 살아 있어야 하는데, 두 번째 줄에서
//   items 를 &mut 로 빌리려고 합니다. 읽는 사람이 있는 동안에는
//   쓰는 사람이 있을 수 없습니다.
//
// 이 규칙이 막아 주는 실제 사고:
//   Vec 은 용량이 부족하면 더 큰 공간을 새로 잡고 데이터를 옮깁니다.
//   그러면 `first` 가 가리키던 주소는 이미 해제된 메모리가 됩니다.

pub fn first_then_push(items: &mut Vec<String>) -> String {
    let first = items[0].clone();
    items.push("added".to_string());
    first
}
