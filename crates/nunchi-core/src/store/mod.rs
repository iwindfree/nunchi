//! 저장 계층 어댑터 (PLAN.md 2절)
//!
//! **이 트레이트가 좁게 유지되는 것이 설계의 핵심이다.** v1은 SQLite로 가지만
//! 엔진 스파이크 결과에 따라 LadybugDB 등으로 갈아탈 수 있어야 하며,
//! 그 교체 비용을 하루 이내로 묶는 장치가 이 6개 메서드다.

pub mod sqlite;

use crate::model::{Direction, Edge, EdgeKind, Node, NodeId};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub node: Node,
    /// BM25 점수. 클수록 관련성이 높다(SQLite bm25()의 부호를 뒤집은 값).
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct Ranked {
    pub node: Node,
    pub score: f32,
    /// 점수 기여 내역 — TUI 팩 미리보기와 `why` 필드에 그대로 쓰인다.
    pub contributions: Vec<(&'static str, f32)>,
}

#[derive(Debug, Clone)]
pub struct RankOpts {
    pub weights: crate::config::RankWeights,
    pub depth: u32,
    pub limit: usize,
}

/// 그래프 저장소. 구현체는 `sqlite::SqliteStore`.
pub trait Store {
    fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize>;

    fn upsert_edges(&mut self, edges: &[Edge]) -> Result<usize>;

    /// `depth` 홉 이내의 이웃. `kinds`가 비어 있으면 모든 엣지 종류를 따른다.
    fn neighbors(
        &self,
        id: &NodeId,
        kinds: &[EdgeKind],
        dir: Direction,
        depth: u32,
    ) -> Result<Vec<Node>>;

    /// `from`에서 `to`까지의 최단 경로들.
    fn paths(&self, from: &NodeId, to: &NodeId, max_depth: u32) -> Result<Vec<Vec<NodeId>>>;

    /// 전문 검색 (BM25).
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>>;

    /// 시드 기준 랭킹. Personalized PageRank는 메모리 인접리스트에서 계산한다
    /// (PLAN.md 3.6절 — C 계층을 쓰기 경로에서 제외).
    fn rank(&self, seeds: &[NodeId], opts: &RankOpts) -> Result<Vec<Ranked>>;
}
