// 정답
//
// pub 을 붙이는 자리가 두 곳입니다. 타입 자체와 각 필드입니다.
// 타입만 pub 이고 필드가 아니면 밖에서 필드를 읽을 수 없습니다(7.1장).
//
// 열거형은 값(variant)에 따로 pub 을 붙이지 않습니다. 열거형이 공개되면
// 그 값들도 함께 공개됩니다.

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, PartialEq)]
pub enum Provenance {
    Fast,
    Precise,
}

pub fn overlaps(a: &Span, b: &Span) -> bool {
    a.start_line <= b.end_line && b.start_line <= a.end_line
}
