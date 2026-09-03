// 정답
//
// matches! 는 "이 값이 이 모양인가" 를 참과 거짓으로 돌려줍니다.
// true 와 false 를 적는 갈래가 사라지므로 한 줄이 됩니다.
//
// 두 번째 함수의 가드가 실제로 하는 일:
//   nunchi 의 mapper_xml.rs 가 <select 를 찾을 때 이 판정을 씁니다.
//   <selectKey> 의 'K' 는 공백도 '>' 도 아니므로 걸러집니다.
//   이 검사가 없으면 selectKey 를 select 문으로 잘못 세게 됩니다.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeKind {
    Symbol,
    File,
    Route,
    Commit,
    Author,
}

pub fn is_packable(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::Symbol | NodeKind::File | NodeKind::Route)
}

pub fn is_tag_boundary(next: Option<char>) -> bool {
    matches!(next, Some(c) if c.is_whitespace() || c == '>')
}
