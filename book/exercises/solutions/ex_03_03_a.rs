// 정답
//
// 세 값을 나란히 꺼냅니다. 들여쓰기가 한 단계도 늘지 않습니다.
//
// 12줄이 4줄로 줄었고, 읽는 사람은 "세 값이 모두 있어야 아래로 내려간다" 는
// 사실을 한눈에 봅니다.
//
// else 블록에서 반드시 빠져나가야 하는 이유:
//   else 를 지나 아래로 내려가면 path 가 없는 상태가 되는데, 아래 코드는
//   path 가 있다고 가정합니다. 컴파일러가 그 상황을 막습니다.

pub struct Node {
    pub path: Option<String>,
    pub span: Option<(u32, u32)>,
    pub lang: Option<String>,
}

pub fn location(node: &Node) -> Option<String> {
    let Some(path) = node.path.as_deref() else { return None };
    let Some((start, end)) = node.span else { return None };
    let Some(lang) = node.lang.as_deref() else { return None };
    Some(format!("[{lang}] {path}:{start}-{end}"))
}
