// 정답
//
//     if let Some(_) = node.span { stats.with_span += 1; }
//     if let Some(_) = node.doc  { stats.with_doc += 1; }
//
// None 일 때 할 일이 없으므로 그 갈래를 적을 이유가 없습니다.
//
// 참고: 이 경우에는 값을 꺼내 쓰지도 않으므로 .is_some() 이 더 간결합니다.
//
//     if node.span.is_some() { stats.with_span += 1; }
//
// 이 문제에서는 if let 을 연습하기 위해 그쪽으로 지정했습니다.
// 실제 코드에서는 값을 꺼내 쓸 때 if let 을, 있는지만 볼 때 is_some 을 씁니다.

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
    if let Some(_) = node.span {
        stats.with_span += 1;
    }
    if let Some(_) = &node.doc {
        stats.with_doc += 1;
    }
}
