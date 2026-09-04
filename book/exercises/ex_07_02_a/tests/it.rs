// 이 파일은 문제의 테스트가 실제로 작성되었는지 확인합니다.
// src/lib.rs 안에 #[cfg(test)] mod tests 를 작성하셔야 합니다.

#[test]
fn unit_tests_must_exist() {
    let src = include_str!("../src/lib.rs");
    assert!(
        src.contains("#[cfg(test)]"),
        "src/lib.rs 에 #[cfg(test)] 모듈이 없습니다"
    );
    assert!(
        src.contains("use super::*"),
        "테스트 모듈에서 use super::*; 로 바깥 항목을 가져오십시오"
    );
    let test_count = src.matches("#[test]").count();
    assert!(
        test_count >= 2,
        "#[test] 함수가 두 개 이상 있어야 합니다 (현재 {test_count}개)"
    );
}
