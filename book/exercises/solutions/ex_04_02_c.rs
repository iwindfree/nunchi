// 정답
//
//     files.iter().filter(|f| f.is_code).map(|f| f.lines).sum()
//
// 반복문 방식과 무엇이 다른가:
//
// 1. mut 변수가 사라집니다. 중간 상태를 추적할 필요가 없습니다.
// 2. 무엇을 하는지가 이름으로 드러납니다. filter 는 고르고 sum 은 더합니다.
// 3. 초기값을 잊거나 += 를 = 로 잘못 쓰는 실수가 생길 수 없습니다.
//
// 반환 타입이 usize 로 정해져 있으므로 sum() 이 무엇을 만들지 알 수 있습니다.
// 그렇지 않으면 sum::<usize>() 처럼 알려 줘야 합니다(4.3장).

pub struct FileInfo {
    pub path: String,
    pub lines: usize,
    pub is_code: bool,
}

pub fn total_code_lines(files: &[FileInfo]) -> usize {
    files.iter().filter(|f| f.is_code).map(|f| f.lines).sum()
}
