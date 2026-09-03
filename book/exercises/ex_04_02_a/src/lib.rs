// 아래 함수는 컴파일되지 않습니다.
//
// into_iter() 가 목록의 소유권을 가져가서, 그 뒤에 목록을 쓸 수 없습니다.
// 이 함수는 길이만 재고 목록을 보관하지 않습니다.
//
// 한 낱말만 바꿔서 고치십시오(4.2장).

pub fn summarize(names: Vec<String>) -> (usize, usize) {
    let total: usize = names.into_iter().map(|n| n.len()).sum();
    (names.len(), total) // TODO: names 를 쓸 수 없습니다
}
