// 아래 함수는 컴파일되지 않습니다.
//
// `total`에 값을 더하려고 하는데 컴파일러가 거부합니다.
// 오류 메시지를 읽고 한 글자만 추가해서 고치십시오.
//
// 힌트: 오류 메시지에 "help:" 로 시작하는 줄이 있습니다. 그것이 답입니다.

pub fn sum_lines(counts: &[u32]) -> u32 {
    let total = 0;
    for c in counts {
        total = total + c; // TODO: 이 줄에서 오류가 납니다
    }
    total
}
