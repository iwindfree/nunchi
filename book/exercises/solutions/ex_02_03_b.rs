// 정답: 반환 타입을 Option<String> 으로 바꿉니다.
//
// ? 를 Option 에 쓰려면 함수도 Option 을 돌려줘야 합니다.
// 값이 없으면 그 자리에서 None 을 돌려주고 함수를 끝냅니다.
//
// Result 와 Option 을 섞을 수는 없습니다. Result 를 돌려주는 함수 안에서
// Option 에 ? 를 붙이면 컴파일되지 않습니다. 그럴 때는 .ok_or() 로
// 먼저 변환합니다.

pub fn extension_lower(path: &str) -> Option<String> {
    let file = path.rsplit('/').next()?;
    let (_, ext) = file.rsplit_once('.')?;
    Some(ext.to_ascii_lowercase())
}
