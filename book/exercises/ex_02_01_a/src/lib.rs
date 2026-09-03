// 아래 함수는 컴파일되지 않습니다.
//
// 확장자가 없는 파일도 있으므로 항상 문자열을 돌려줄 수는 없습니다.
// 서명을 고쳐서 "없을 수도 있다" 는 사실을 타입에 적으십시오.
//
// 본문은 그대로 두십시오.

pub fn extension(path: &str) -> String {  // TODO: 반환 타입이 틀렸습니다
    path.rsplit_once('.').map(|(_, ext)| ext.to_string())
}

pub fn language_of(path: &str) -> String { // TODO: 반환 타입이 틀렸습니다
    match extension(path).as_deref() {
        Some("rs") => Some("rust"),
        Some("java") => Some("java"),
        _ => None,
    }
}
