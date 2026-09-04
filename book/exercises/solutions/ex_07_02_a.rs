// 정답 예시
//
// #[cfg(test)] 는 "테스트를 컴파일할 때만 이 코드를 포함한다" 는 뜻입니다.
// 배포용 빌드에는 들어가지 않으므로 바이너리가 커지지 않습니다.
//
// use super::* 는 부모 모듈의 항목을 전부 가져옵니다. 테스트 모듈이
// 파일 안쪽에 있으므로 바깥 함수를 그냥 쓸 수 없고 가져와야 합니다.
//
// 이 배치의 값어치는 비공개 함수도 테스트할 수 있다는 점입니다.
// tests/ 디렉터리에 있는 테스트는 밖에서 부르는 것이므로 pub 인 것만
// 볼 수 있지만, 같은 파일 안의 테스트는 전부 볼 수 있습니다.
//
// nunchi 의 거의 모든 모듈이 이 구조입니다.

pub fn extension(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next()?;
    name.rsplit_once('.').map(|(_, ext)| ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_extension() {
        assert_eq!(extension("src/main.rs"), Some("rs"));
        assert_eq!(extension("a/b/App.java"), Some("java"));
    }

    #[test]
    fn handles_missing_extension() {
        assert_eq!(extension("Makefile"), None);
        assert_eq!(extension(""), None);
    }
}
