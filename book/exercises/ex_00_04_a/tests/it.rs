use ex_00_04_a::{overlaps, Provenance, Span};

#[test]
fn span_holds_two_lines() {
    let s = Span { start_line: 10, end_line: 25 };
    assert_eq!(s.start_line, 10);
    assert_eq!(s.end_line, 25);
}

#[test]
fn spans_can_overlap() {
    let a = Span { start_line: 10, end_line: 25 };
    let b = Span { start_line: 20, end_line: 30 };
    let c = Span { start_line: 40, end_line: 50 };
    assert!(overlaps(&a, &b));
    assert!(!overlaps(&a, &c));
}

#[test]
fn provenance_has_two_values() {
    let fast = Provenance::Fast;
    let precise = Provenance::Precise;
    assert_ne!(fast, precise);
}
