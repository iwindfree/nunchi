// nunchi 가 엣지를 만드는 상황과 같습니다.
//
// 파일 하나에서 심볼 여러 개를 찾았고, 각 심볼마다 "파일이 심볼을 담는다"는
// 엣지를 만들어야 합니다. 엣지는 NodeId 를 소유해야 합니다.
//
// 아래 코드는 컴파일되지 않습니다. 반복 안에서 file_id 를 계속 써야 하는데
// 첫 번째 반복에서 소유권이 넘어가기 때문입니다.
//
// 고치십시오. 이번에는 clone 을 쓰는 것이 정답입니다.
// 어디에 붙여야 하는지 생각해 보십시오.

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
        edges.push(Edge { src: file_id, dst: sym }); // TODO: 오류가 납니다
    }
    edges
}
