// 아래 함수는 컴파일되지 않습니다.
//
// 파일 크기를 받아서 사람이 읽기 좋은 문자열로 바꾸려고 합니다.
// `size`가 숫자였는데 문자열로 바뀌어야 합니다.
//
// `mut`을 붙이는 것으로는 해결되지 않습니다. 타입이 달라지기 때문입니다.
// 0.2장의 "변수 가리기"를 다시 읽어 보십시오.

pub fn describe_size(bytes: u64) -> String {
    let size = bytes;
    size = format!("{} bytes", size); // TODO: 이 줄에서 오류가 납니다
    size
}
