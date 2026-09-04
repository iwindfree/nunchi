use ex_06_01_a::longest;

#[test]
fn works_with_vec() {
    let v = vec!["a".to_string(), "bbb".to_string(), "cc".to_string()];
    assert_eq!(longest(&v).map(String::as_str), Some("bbb"));
}

#[test]
fn works_with_array() {
    // &[String; 2] 가 &[String] 으로 자동 변환됩니다
    let arr = ["xx".to_string(), "y".to_string()];
    assert_eq!(longest(&arr).map(String::as_str), Some("xx"));
}

#[test]
fn works_with_slice() {
    let v = vec!["a".to_string(), "bbbb".to_string(), "cc".to_string()];
    assert_eq!(longest(&v[1..]).map(String::as_str), Some("bbbb"));
}
