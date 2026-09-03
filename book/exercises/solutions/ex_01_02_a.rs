// 정답: derive 에 Clone 과 Copy 를 추가합니다.
//
//     #[derive(Debug, PartialEq, Clone, Copy)]
//
// Copy 를 붙이려면 Clone 도 있어야 합니다. Copy 는 Clone 의 특수한 경우이며
// "복사가 저렴하므로 자동으로 해도 된다"는 표시입니다.
//
// Span 은 u32 두 개라서 8바이트입니다. 이 정도는 참조를 넘기는 것과
// 비용이 비슷하므로 Copy 가 맞습니다.
//
// 반대로 String 이 들어 있는 구조체에는 Copy 를 붙일 수 없습니다.
// 힙 데이터를 복사해야 하므로 무료가 아니고, Rust 는 비용이 드는 일을
// 몰래 하지 않습니다.

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

pub fn line_count(span: Span) -> u32 {
    span.end_line - span.start_line + 1
}

pub fn describe(span: Span) -> String {
    let n = line_count(span);
    format!("{}-{} ({} lines)", span.start_line, span.end_line, n)
}
