use ex_05_01_a::Span;

#[test]
fn creates_span() {
    let s = Span::new(88, 141);
    assert_eq!(s.start_line, 88);
    assert_eq!(s.end_line, 141);
}

#[test]
fn counts_lines_inclusively() {
    assert_eq!(Span::new(10, 10).line_count(), 1);
    assert_eq!(Span::new(88, 141).line_count(), 54);
}
