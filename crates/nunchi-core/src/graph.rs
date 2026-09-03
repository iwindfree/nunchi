//! 메모리 상주 그래프 — 순회와 랭킹 (docs/DESIGN.md 7절·3.6절)
//!
//! **순회 엔진은 저장 엔진이 아니다.** 엣지 100만 개는 메모리에 ~50MB로 들어가므로
//! PPR·중심성은 여기서 계산한다. 이 설계 덕분에 저장 계층이 SQLite여도
//! 그래프 DB의 순회 이점이 상쇄된다.
//!
//! 또한 Personalized PageRank는 **시드 의존적이라 미리 계산할 수 없다.**
//! 그래서 C 계층(전역 파생값)을 쓰기 경로에서 뺄 수 있었다(docs/DESIGN.md 8절).

use crate::model::{EdgeKind, NodeId};
use crate::store::sqlite::SqliteStore;
use anyhow::Result;
use std::collections::HashMap;

pub struct MemGraph {
    ids: Vec<NodeId>,
    index: HashMap<String, usize>,
    /// (대상, 가중치) — 가중치는 confidence × weight
    out: Vec<Vec<(usize, f32)>>,
    inc: Vec<Vec<(usize, f32)>>,
    kinds: Vec<Vec<(usize, EdgeKind)>>,
}

impl MemGraph {
    pub fn load(store: &SqliteStore) -> Result<Self> {
        let node_ids = store.all_node_ids()?;
        let mut index = HashMap::with_capacity(node_ids.len());
        for (i, id) in node_ids.iter().enumerate() {
            index.insert(id.0.clone(), i);
        }

        let n = node_ids.len();
        let mut graph = MemGraph {
            ids: node_ids,
            index,
            out: vec![Vec::new(); n],
            inc: vec![Vec::new(); n],
            kinds: vec![Vec::new(); n],
        };

        for (src, dst, kind, weight) in store.all_edges()? {
            let (Some(&s), Some(&d)) = (graph.index.get(&src), graph.index.get(&dst)) else {
                continue;
            };
            graph.out[s].push((d, weight));
            graph.inc[d].push((s, weight));
            if let Some(k) = EdgeKind::parse(&kind) {
                graph.kinds[s].push((d, k));
            }
        }
        Ok(graph)
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn index_of(&self, id: &NodeId) -> Option<usize> {
        self.index.get(&id.0).copied()
    }

    pub fn id_at(&self, i: usize) -> &NodeId {
        &self.ids[i]
    }
}

/// 기본 damping. 실측으로 정한 값이다.
///
/// 엣지를 무향으로 다루므로 damping이 높으면 **차수 높은 이웃이 시드를 추월한다**
/// (경로 a→b→c 에서 d=0.85 이면 b=0.46 > a=0.35). 컨텍스트 랭킹에서는
/// 질의가 직접 짚은 시드가 가장 높아야 하므로 재시작 확률을 크게 잡는다.
/// d=0.5 → 시드 0.58 · 1홉 0.33 · 2홉 0.08 로 거리에 따라 깔끔히 감쇠한다.
pub const DEFAULT_DAMPING: f32 = 0.5;

impl MemGraph {
    /// 시드에서 출발하는 Personalized PageRank.
    ///
    /// 엣지를 무향으로 다룬다 — "이 심볼을 호출하는 쪽"도 "호출당하는 쪽"만큼
    /// 컨텍스트로 중요하기 때문이다.
    pub fn personalized_pagerank(&self, seeds: &[usize], damping: f32, iterations: usize) -> Vec<f32> {
        let n = self.ids.len();
        let mut rank = vec![0.0f32; n];
        if seeds.is_empty() || n == 0 {
            return rank;
        }

        let seed_mass = 1.0 / seeds.len() as f32;
        let mut restart = vec![0.0f32; n];
        for &s in seeds {
            if s < n {
                restart[s] += seed_mass;
            }
        }
        rank.copy_from_slice(&restart);

        let mut next = vec![0.0f32; n];
        for _ in 0..iterations {
            next.iter_mut().for_each(|v| *v = 0.0);

            for i in 0..n {
                if rank[i] == 0.0 {
                    continue;
                }
                let total: f32 = self.out[i].iter().map(|(_, w)| *w).sum::<f32>()
                    + self.inc[i].iter().map(|(_, w)| *w).sum::<f32>();
                if total <= 0.0 {
                    // 고립 노드의 질량은 시드로 되돌린다.
                    for (j, r) in restart.iter().enumerate() {
                        next[j] += rank[i] * r;
                    }
                    continue;
                }
                let share = rank[i] * damping / total;
                for (j, w) in &self.out[i] {
                    next[*j] += share * w;
                }
                for (j, w) in &self.inc[i] {
                    next[*j] += share * w;
                }
            }

            for (j, r) in restart.iter().enumerate() {
                next[j] += (1.0 - damping) * r;
            }
            std::mem::swap(&mut rank, &mut next);
        }
        rank
    }

    /// 무가중 도수 중심성(정규화). 허브 심볼을 살짝 끌어올리는 용도다.
    pub fn degree_centrality(&self) -> Vec<f32> {
        let max = self
            .out
            .iter()
            .zip(&self.inc)
            .map(|(o, i)| (o.len() + i.len()) as f32)
            .fold(1.0f32, f32::max);
        self.out
            .iter()
            .zip(&self.inc)
            .map(|(o, i)| (o.len() + i.len()) as f32 / max)
            .collect()
    }

    /// 특정 엣지 종류로만 이웃을 편다.
    pub fn neighbors_of_kind(&self, node: usize, kind: EdgeKind) -> Vec<usize> {
        self.kinds[node]
            .iter()
            .filter(|(_, k)| *k == kind)
            .map(|(d, _)| *d)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::store::Store;

    fn node(id: &str) -> Node {
        Node::new(NodeId(id.into()), NodeKind::Symbol, id, "r")
    }

    #[test]
    fn pagerank_concentrates_near_seeds() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        store.upsert_nodes(&[node("a"), node("b"), node("c"), node("far")])?;
        store.upsert_edges(&[
            Edge::new(NodeId("a".into()), NodeId("b".into()), EdgeKind::Calls, Provenance::Fast),
            Edge::new(NodeId("b".into()), NodeId("c".into()), EdgeKind::Calls, Provenance::Fast),
        ])?;

        let g = MemGraph::load(&store)?;
        let seed = g.index_of(&NodeId("a".into())).unwrap();
        let pr = g.personalized_pagerank(&[seed], DEFAULT_DAMPING, 30);

        let at = |id: &str| pr[g.index_of(&NodeId(id.into())).unwrap()];
        // 시드가 가장 높고, 거리가 멀수록 낮아지며, 비연결 노드는 거의 0이다.
        assert!(at("a") > at("b"), "a={} b={}", at("a"), at("b"));
        assert!(at("b") > at("c"), "b={} c={}", at("b"), at("c"));
        assert!(at("c") > at("far"), "c={} far={}", at("c"), at("far"));
        Ok(())
    }

    #[test]
    fn empty_seeds_give_zero_rank() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        store.upsert_nodes(&[node("a")])?;
        let g = MemGraph::load(&store)?;
        assert!(g.personalized_pagerank(&[], 0.85, 5).iter().all(|v| *v == 0.0));
        Ok(())
    }
}
