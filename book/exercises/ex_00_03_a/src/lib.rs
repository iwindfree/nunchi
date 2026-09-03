// 아래 두 함수는 컴파일되지 않습니다.
//
// 함수 서명에 적힌 타입과 실제로 돌려주는 값의 타입이 맞지 않습니다.
// 서명을 고치십시오. 함수 본문은 그대로 두십시오.
//
// 힌트: 오류 메시지의 "expected ..., found ..." 를 읽으십시오.
//       왼쪽이 서명에 적힌 것이고 오른쪽이 실제 값입니다.

/// 파일 경로에서 확장자만 꺼냅니다. 확장자가 없으면 None 입니다.
pub fn extension(path: &str) -> String {   // TODO: 반환 타입이 틀렸습니다
    path.rsplit_once('.').map(|(_, ext)| ext.to_string())
}

/// 줄 수를 셉니다.
pub fn line_count(text: &str) -> String {  // TODO: 반환 타입이 틀렸습니다
    text.lines().count()
}
