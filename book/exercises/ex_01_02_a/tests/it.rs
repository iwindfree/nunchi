use ex_01_02_a::{describe, line_count, Span};

#[test]
fn counts_lines_in_span() {
    let s = Span { start_line: 10, end_line: 12 };
    assert_eq!(line_count(s), 3);
    // s 가 Copy 이므로 위에서 넘긴 뒤에도 쓸 수 있습니다
    assert_eq!(s.start_line, 10);
}

#[test]
fn describes_span() {
    let s = Span { start_line: 88, end_line: 141 };
    assert_eq!(describe(s), "88-141 (54 lines)");
}
