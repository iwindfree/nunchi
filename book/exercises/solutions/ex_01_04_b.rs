// 정답: src 에 clone 을 붙입니다.
//
//     edges.push(Edge { src: file_id.clone(), dst: sym });
//
// 이 clone 은 정당합니다. 이유가 세 가지입니다.
//
// 1. Edge 가 NodeId 를 소유해야 합니다.
//    빌린 참조를 넣으면 수명 표기가 필요해지고, 이 함수 밖으로 Vec<Edge> 를
//    돌려줄 수 없게 됩니다(1.6장).
//
// 2. 반복마다 새 Edge 가 만들어지므로 각자 자기 NodeId 가 필요합니다.
//
// 3. NodeId 는 짧은 문자열 하나이므로 복사 비용이 작습니다.
//
// dst 에는 clone 이 없습니다. sym 은 반복마다 새로 받는 값이고 그 자리에서
// 소유권을 넘기면 되기 때문입니다. 마지막 사용에서는 넘기는 것이 맞습니다.
//
// nunchi 의 index.rs 에 똑같은 형태가 있습니다.
//
//     edges.push(Edge::new(
//         repo_id.clone(),   // 반복마다 필요하므로 복사합니다
//         file_id,           // 이 자리에서 끝이므로 넘깁니다
//         ...

#[derive(Debug, Clone, PartialEq)]
pub struct NodeId(pub String);

#[derive(Debug, PartialEq)]
pub struct Edge {
    pub src: NodeId,
    pub dst: NodeId,
}

pub fn contains_edges(file_id: NodeId, symbols: Vec<NodeId>) -> Vec<Edge> {
    let mut edges = Vec::new();
    for sym in symbols {
        edges.push(Edge { src: file_id.clone(), dst: sym });
    }
    edges
}
