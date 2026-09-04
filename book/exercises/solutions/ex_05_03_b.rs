// 정답
//
// From 을 구현하면 ? 가 알아서 부릅니다. 함수 본문을 고칠 필요가 없습니다.
//
// ? 가 하는 일을 풀어 쓰면 이렇습니다.
//
//     let start: u32 = match a.parse() {
//         Ok(v) => v,
//         Err(e) => return Err(SpanError::from(e)),   여기서 From 을 부릅니다
//     };
//
// 이 장치 덕분에 함수마다 오류 타입이 달라도 ? 하나로 이어 붙일 수 있습니다.
// nunchi 가 anyhow::Result 를 쓰면서 rusqlite, serde_json, std::io 의
// 오류를 전부 ? 로 처리하는 이유가 이것입니다.

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

impl From<std::num::ParseIntError> for SpanError {
    fn from(_: std::num::ParseIntError) -> SpanError {
        SpanError::BadNumber
    }
}

pub fn parse_span(text: &str) -> Result<Span, SpanError> {
    let (a, b) = text.split_once('-').ok_or(SpanError::BadFormat)?;
    let start: u32 = a.parse()?;
    let end: u32 = b.parse()?;
    Ok(Span { start, end })
}
