// 정답
//
// extension  -> Option<String>
// language_of -> Option<&'static str>
//
// language_of 의 반환 타입에 'static 이 붙는 이유:
//   "rust" 와 "java" 는 코드에 직접 쓴 문자열이므로 프로그램이 끝날 때까지
//   살아 있습니다. 그것을 &'static str 이라고 적습니다(1.6장).
//   새 String 을 만들 필요가 없으므로 힙 할당이 없습니다.
//
// nunchi 의 lang.rs 가 정확히 이 형태입니다.
//
//     pub fn detect(path: &Path) -> Option<&'static str>

pub fn extension(path: &str) -> Option<String> {
    path.rsplit_once('.').map(|(_, ext)| ext.to_string())
}

pub fn language_of(path: &str) -> Option<&'static str> {
    match extension(path).as_deref() {
        Some("rs") => Some("rust"),
        Some("java") => Some("java"),
        _ => None,
    }
}
