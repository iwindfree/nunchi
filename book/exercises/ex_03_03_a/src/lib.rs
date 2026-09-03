// 아래 함수는 동작하지만 if let 이 세 겹으로 중첩되어 읽기 어렵습니다.
//
// let ... else 로 펴십시오. 들여쓰기가 늘지 않아야 합니다(3.3장).
//
// nunchi 의 index.rs 에 실제로 이런 형태가 있었고 같은 방식으로 고쳤습니다.

pub struct Node {
    pub path: Option<String>,
    pub span: Option<(u32, u32)>,
    pub lang: Option<String>,
}

pub fn location(node: &Node) -> Option<String> {
    if let Some(path) = node.path.as_deref() {
        if let Some((start, end)) = node.span {
            if let Some(lang) = node.lang.as_deref() {
                Some(format!("[{lang}] {path}:{start}-{end}"))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
