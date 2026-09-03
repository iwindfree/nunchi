// 정답: 두 번째 줄을 `let size = ...` 로 바꿉니다.
//
// 재대입(`size = ...`)은 같은 변수의 값을 바꾸므로 타입이 같아야 합니다.
// 변수 가리기(`let size = ...`)는 새 변수를 만들므로 타입을 바꿀 수 있습니다.
//
// 여기서는 u64 에서 String 으로 타입이 바뀌므로 변수 가리기가 필요합니다.

pub fn describe_size(bytes: u64) -> String {
    let size = bytes;
    let size = format!("{} bytes", size);
    size
}

// 참고: 실제로는 중간 변수 없이 한 줄로 씁니다.
//
//     format!("{bytes} bytes")
