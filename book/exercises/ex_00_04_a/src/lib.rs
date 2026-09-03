// 테스트가 통과하도록 아래 두 타입을 정의하십시오.
//
// 1. `Span` 구조체
//    - 필드 두 개: start_line, end_line (둘 다 u32)
//    - 두 필드 모두 밖에서 읽을 수 있어야 합니다
//
// 2. `Provenance` 열거형
//    - 가능한 값 두 가지: Fast, Precise
//
// 테스트 파일(tests/it.rs)을 보시면 어떻게 쓰이는지 알 수 있습니다.
// nunchi 의 실제 타입과 같은 모양입니다.
//
// 힌트: `#[derive(Debug, PartialEq)]` 를 붙이지 않으면 assert_eq! 가
//       컴파일되지 않습니다. 이것이 무엇인지는 5.4장에서 다룹니다.
//       지금은 그냥 붙이십시오.

// TODO: 여기에 Span 을 정의하십시오

// TODO: 여기에 Provenance 를 정의하십시오

/// 두 span 이 겹치는지 확인합니다. 이 함수는 고치지 마십시오.
pub fn overlaps(a: &Span, b: &Span) -> bool {
    a.start_line <= b.end_line && b.start_line <= a.end_line
}
