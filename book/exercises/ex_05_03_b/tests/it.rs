use ex_05_03_b::{parse_span, Span, SpanError};

#[test]
fn parses_valid_span() {
    assert_eq!(parse_span("88-141"), Ok(Span { start: 88, end: 141 }));
}

#[test]
fn reports_bad_format() {
    assert_eq!(parse_span("88"), Err(SpanError::BadFormat));
}

#[test]
fn reports_bad_number() {
    assert_eq!(parse_span("aa-141"), Err(SpanError::BadNumber));
}
