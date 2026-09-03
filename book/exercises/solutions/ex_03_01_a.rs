// 정답
//
// 세 경우를 명시적으로 적습니다.
//
// `_ => false` 로 묶으면 훨씬 짧지만 그렇게 하지 않는 이유가 있습니다.
// 나중에 EdgeKind 에 Handles 를 추가하면, `_` 가 있는 코드는 조용히
// false 로 처리하고 넘어갑니다. 명시적으로 적어 두면 컴파일 오류가 나므로
// 새 값을 어떻게 분류할지 반드시 결정하게 됩니다.
//
// nunchi 는 엣지 종류를 19개까지 늘리는 동안 이 방식으로 고칠 곳을
// 컴파일러에게서 전부 받았습니다.

#[derive(Debug, Clone, Copy)]
pub enum EdgeKind {
    Calls,
    Injects,
    CallsApi,
    ModifiedBy,
    AuthoredBy,
}

pub fn is_structural(kind: EdgeKind) -> bool {
    match kind {
        EdgeKind::Calls => true,
        EdgeKind::Injects => true,
        EdgeKind::CallsApi => true,
        EdgeKind::ModifiedBy => false,
        EdgeKind::AuthoredBy => false,
    }
}
