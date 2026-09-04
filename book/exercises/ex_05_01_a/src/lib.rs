// nunchi 의 Span 에 메서드를 붙이는 문제입니다.
//
// 아래 두 가지를 impl 블록 안에 작성하십시오.
//
// 1. 연관 함수 `new(start: u32, end: u32) -> Span`
//    self 를 받지 않습니다. Span::new(1, 5) 처럼 부릅니다.
//
// 2. 메서드 `line_count(&self) -> u32`
//    self 를 빌려서 받습니다. span.line_count() 처럼 부릅니다.
//    시작과 끝을 모두 포함하므로 end - start + 1 입니다.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

impl Span {
    // TODO: new 와 line_count 를 여기에 작성하십시오
}
