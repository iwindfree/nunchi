// 아래 코드는 컴파일되지 않습니다.
//
// `Span` 을 두 번 쓰려고 하는데 첫 번째 사용에서 소유권이 넘어갑니다.
// `Span` 은 u32 두 개뿐이라 복사 비용이 거의 없으므로, 이동시키지 말고
// 복사되게 만드는 것이 맞습니다.
//
// `Span` 에 표시 하나를 추가해서 고치십시오.
// 힌트: 1.2장의 "구조체를 Copy 로 만들기" 를 보십시오.
//       Copy 를 붙이려면 Clone 도 함께 필요합니다.

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

pub fn line_count(span: Span) -> u32 {
    span.end_line - span.start_line + 1
}

pub fn describe(span: Span) -> String {
    let n = line_count(span);
    // TODO: 위에서 span 이 이동했으므로 아래 줄에서 오류가 납니다
    format!("{}-{} ({} lines)", span.start_line, span.end_line, n)
}
