// 정답
//
// 두 함수의 차이가 핵심입니다.
//
// `new` 는 self 를 받지 않으므로 연관 함수(associated function)입니다.
//   타입 이름으로 부릅니다: Span::new(1, 5)
//   생성자 역할을 하는 함수는 관례적으로 new 라고 이름 짓습니다.
//
// `line_count` 는 &self 를 받으므로 메서드입니다.
//   값으로 부릅니다: span.line_count()
//   &self 는 self: &Span 의 줄임말입니다.
//
// Span 이 Copy 이므로 self 를 그냥 받아도 되지만, &self 가 관례입니다.
// Copy 가 아닌 타입에서도 같은 모양이 되기 때문입니다.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Span {
        Span { start_line: start, end_line: end }
    }

    pub fn line_count(&self) -> u32 {
        self.end_line - self.start_line + 1
    }
}
