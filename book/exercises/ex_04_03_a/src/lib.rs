// 아래 함수는 컴파일되지 않습니다.
//
// Option 안의 문자열을 숫자로 바꾸려고 하는데, 바꾸는 것이 실패할 수
// 있으므로 결과가 Option<Option<usize>> 가 되어 두 겹입니다.
//
// 한 낱말만 바꿔서 고치십시오(4.3장).

pub fn parse_depth(raw: Option<String>) -> Option<usize> {
    raw.map(|s| s.trim().parse::<usize>().ok())
}
