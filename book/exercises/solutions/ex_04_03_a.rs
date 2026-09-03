// 정답: map 을 and_then 으로 바꿉니다.
//
//     raw.and_then(|s| s.trim().parse::<usize>().ok())
//
// 기준은 하나입니다. 넘긴 함수가 Option 을 돌려주면 and_then 입니다.
//
//   .map     넘긴 함수가 보통 값을 준다  ->  Option<T>
//   .and_then 넘긴 함수가 Option 을 준다 ->  Option<T> (평평해집니다)
//
// map 을 쓰면 Option<Option<usize>> 가 됩니다. 값이 있는데 그 안에 또
// 없을 수도 있다는 뜻이라 다루기 번거롭습니다.
//
// nunchi 의 index.rs 에 같은 형태가 있습니다.
//
//     .ok()
//     .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
//     .map(|d| d.as_secs() as i64)
//
// 두 번째 줄은 Option 을 돌려주므로 and_then, 세 번째 줄은 보통 값을
// 돌려주므로 map 입니다.

pub fn parse_depth(raw: Option<String>) -> Option<usize> {
    raw.and_then(|s| s.trim().parse::<usize>().ok())
}
