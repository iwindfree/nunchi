// 아래 코드는 컴파일되지 않습니다.
//
// nunchi 의 실제 상황과 같습니다. 파일 경로 하나로 두 가지를 만들어야 합니다.
// 파일 노드의 id 와, 그 파일을 가리키는 엣지입니다.
//
// `make_file_id` 가 경로의 소유권을 가져가 버려서 그 뒤에 경로를 쓸 수
// 없습니다.
//
// `make_file_id` 의 서명을 고쳐서 소유권을 가져가지 않게 만드십시오.
// 함수 본문과 `build` 함수는 그대로 두십시오.

pub fn make_file_id(path: String) -> String {
    format!("file:{}", path)
}

pub fn build(path: String) -> (String, String) {
    let id = make_file_id(path);
    let edge = format!("contains:{}", path); // TODO: 이 줄에서 오류가 납니다
    (id, edge)
}
