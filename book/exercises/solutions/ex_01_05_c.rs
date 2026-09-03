// 정답
//
// 매개변수를 &str 로 받는 이유: 세 함수 모두 값을 보관하지 않고 새 문자열을
// 만들어 돌려줍니다. 소유권을 받을 이유가 없습니다.
//
// 반환값이 String 인 이유: 함수 안에서 새로 만든 문자열이므로 함수가 끝나면
// 사라집니다. 빌려서 돌려줄 수 없습니다(1.5장).

pub fn repo_id(repo: &str) -> String {
    format!("repo:{repo}")
}

pub fn file_id(repo: &str, path: &str) -> String {
    format!("file:{repo}/{path}")
}

pub fn symbol_id(repo: &str, path: &str, name: &str) -> String {
    format!("sym:{repo}/{path}#{name}")
}
