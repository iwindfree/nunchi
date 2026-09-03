// nunchi 의 lang.rs 를 단순하게 만든 것입니다.
//
// 아래 함수는 컴파일되지 않습니다. Option 에 ? 를 쓰려고 하는데
// 반환 타입이 맞지 않습니다.
//
// 서명을 고치십시오. 본문은 그대로 두십시오.
//
// 힌트: ? 는 Option 을 돌려주는 함수 안에서 Option 에 쓸 수 있습니다(2.3장).

/// 파일 이름에서 확장자를 소문자로 꺼냅니다.
pub fn extension_lower(path: &str) -> String {   // TODO: 반환 타입이 틀렸습니다
    let file = path.rsplit('/').next()?;
    let (_, ext) = file.rsplit_once('.')?;
    Some(ext.to_ascii_lowercase())
}
