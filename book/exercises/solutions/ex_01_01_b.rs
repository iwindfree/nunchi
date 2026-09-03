// 정답: `make_file_id` 가 `&str` 을 받게 바꿉니다.
//
//     pub fn make_file_id(path: &str) -> String
//
// 그리고 호출할 때 `&path` 로 빌려서 넘깁니다.
//
// 왜 String 이 아니라 &str 인가:
//   이 함수는 경로를 읽어서 새 문자열을 만들 뿐이고 원본을 보관하지 않습니다.
//   그런 함수는 소유권을 받을 이유가 없습니다. 빌리기만 하면 충분합니다.
//
//   "값을 보관하지 않는 함수는 빌려 받는다"가 Rust 의 일반 관례입니다.
//   nunchi 코드의 함수 대부분이 &str 이나 &T 를 받는 이유가 이것입니다.

pub fn make_file_id(path: &str) -> String {
    format!("file:{}", path)
}

pub fn build(path: String) -> (String, String) {
    let id = make_file_id(&path);
    let edge = format!("contains:{}", path);
    (id, edge)
}
