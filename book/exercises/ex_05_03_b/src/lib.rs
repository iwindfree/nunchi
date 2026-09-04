// ? 연산자가 오류 타입을 자동 변환하는 것을 확인하는 문제입니다.
//
// `parse_span` 은 "88-141" 형식의 문자열을 Span 으로 바꿉니다.
// 숫자 변환이 실패하면 ParseIntError 가 나는데, 이 함수는
// SpanError 를 돌려주어야 합니다.
//
// ? 가 자동으로 변환하게 하려면 무엇이 필요합니까?
//
// 힌트: From<std::num::ParseIntError> for SpanError 를 구현하십시오.

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, PartialEq)]
pub enum SpanError {
    BadFormat,
    BadNumber,
}

// TODO: 여기에 From 구현을 작성하십시오

pub fn parse_span(text: &str) -> Result<Span, SpanError> {
    let (a, b) = text.split_once('-').ok_or(SpanError::BadFormat)?;
    // 아래 두 ? 가 ParseIntError 를 SpanError 로 변환해야 합니다
    let start: u32 = a.parse()?;
    let end: u32 = b.parse()?;
    Ok(Span { start, end })
}
