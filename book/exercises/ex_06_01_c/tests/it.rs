use ex_06_01_c::top_n;

fn sample() -> Vec<(String, f32)> {
    vec![
        ("low".to_string(), 0.1),
        ("high".to_string(), 0.9),
        ("mid".to_string(), 0.5),
    ]
}

#[test]
fn takes_top_two() {
    assert_eq!(top_n(sample(), 2), vec!["high", "mid"]);
}

#[test]
fn handles_short_list() {
    // 3개뿐인데 10개를 요청해도 멈추지 않아야 합니다
    assert_eq!(top_n(sample(), 10), vec!["high", "mid", "low"]);
}

#[test]
fn handles_zero() {
    assert!(top_n(sample(), 0).is_empty());
}
