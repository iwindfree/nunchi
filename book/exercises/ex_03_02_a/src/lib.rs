// 아래 코드는 동작하지만 아무 일도 하지 않는 갈래를 적고 있습니다.
//
// 두 match 를 if let 으로 바꾸십시오(3.2장).

#[derive(Debug, Default)]
pub struct Stats {
    pub with_span: usize,
    pub with_doc: usize,
}

pub struct Node {
    pub span: Option<(u32, u32)>,
    pub doc: Option<String>,
}

pub fn tally(node: &Node, stats: &mut Stats) {
    match node.span {
        Some(_) => stats.with_span += 1,
        None => {}
    }
    match &node.doc {
        Some(_) => stats.with_doc += 1,
        None => {}
    }
}
