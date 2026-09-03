// 아래 함수는 동작하지만 반복문과 mut 변수를 씁니다.
//
// 이터레이터 체인으로 바꾸십시오. mut 변수가 하나도 남지 않아야 합니다.
//
// 힌트: 코드 파일만 골라서 그 줄 수를 모두 더합니다.
//       filter 와 map 과 sum 을 쓰면 됩니다(4.2장, 4.3장).

pub struct FileInfo {
    pub path: String,
    pub lines: usize,
    pub is_code: bool,
}

pub fn total_code_lines(files: &[FileInfo]) -> usize {
    let mut total = 0;
    for f in files {
        if f.is_code {
            total += f.lines;
        }
    }
    total
}
