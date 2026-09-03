use ex_04_03_d::parse_all;

#[test]
fn parses_valid_input() {
    let good = vec!["1".to_string(), " 2 ".to_string(), "3".to_string()];
    assert_eq!(parse_all(&good).unwrap(), vec![1, 2, 3]);
}

#[test]
fn reports_failure_instead_of_dropping_it() {
    let bad = vec!["1".to_string(), "two".to_string(), "3".to_string()];
    assert!(
        parse_all(&bad).is_err(),
        "잘못된 값을 말없이 버리면 안 됩니다. 하나라도 실패하면 전체가 실패해야 합니다"
    );
}

#[test]
fn empty_input_succeeds() {
    assert_eq!(parse_all(&[]).unwrap(), Vec::<u32>::new());
}
