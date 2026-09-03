// 아래 두 함수를 matches! 로 한 줄씩 줄이십시오(3.3장).
//
// 두 번째 함수에는 가드가 필요합니다.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeKind {
    Symbol,
    File,
    Route,
    Commit,
    Author,
}

/// 팩에 담을 수 있는 종류인지 봅니다.
pub fn is_packable(kind: NodeKind) -> bool {
    match kind {
        NodeKind::Symbol | NodeKind::File | NodeKind::Route => true,
        _ => false,
    }
}

/// 다음 글자가 태그 경계인지 봅니다. 공백이거나 '>' 여야 합니다.
pub fn is_tag_boundary(next: Option<char>) -> bool {
    match next {
        Some(c) if c.is_whitespace() || c == '>' => true,
        _ => false,
    }
}
