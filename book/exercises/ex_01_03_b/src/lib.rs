// 아래 코드는 컴파일되지 않습니다.
//
// `total_length` 가 목록의 소유권을 가져가 버려서, 그 뒤에 목록을 쓸 수
// 없습니다. 이 함수는 길이만 재고 목록을 보관하지 않습니다.
//
// `total_length` 의 서명을 고치십시오. `summarize` 는 그대로 두십시오.
//
// 힌트: 값을 보관하지 않는 함수는 빌려 받습니다(1.3장).
//       Vec<String> 을 빌리는 표기는 &[String] 입니다(6.1장에서 자세히 다룹니다).

pub fn total_length(names: Vec<String>) -> usize {
    names.iter().map(|n| n.len()).sum()
}

pub fn summarize(names: Vec<String>) -> String {
    let total = total_length(names);
    format!("{} names, {} chars", names.len(), total) // TODO: 오류가 납니다
}
