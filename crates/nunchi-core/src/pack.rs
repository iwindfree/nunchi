//! 컨텍스트 팩 — 토큰 절감의 본체 (PLAN.md 3절·3.5절)
//!
//! 에이전트에게 **답이 아니라 좌표**를 준다. 파일 12개 전체(≈35k) 대신
//! 심볼 40개를 L0/L1/L2 혼합으로 렌더링(≈4k)하고 정확한 `path:line`을 붙인다.
//! 60k를 4k로 만드는 실체는 이 "선별 + 강등" 두 동작이다.

use crate::config::RankWeights;
use crate::graph::{MemGraph, DEFAULT_DAMPING};
use crate::model::{EdgeKind, Node, NodeId, NodeKind};
use crate::path as npath;
use crate::store::{sqlite::SqliteStore, Store};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

/// 렌더링 상세도. 예산이 줄면 L2 → L1 → L0로 강등된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tier {
    /// 시그니처 한 줄 + 좌표
    L0,
    /// 시그니처 + 문서 + 핵심 몇 줄
    L1,
    /// 전체 본문
    L2,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::L0 => "L0",
            Tier::L1 => "L1",
            Tier::L2 => "L2",
        }
    }
    fn lower(self) -> Option<Tier> {
        match self {
            Tier::L2 => Some(Tier::L1),
            Tier::L1 => Some(Tier::L0),
            Tier::L0 => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PackItem {
    pub tier: &'static str,
    /// `path:line` 또는 `path:start-end` — 에이전트가 필요할 때 이 범위만 Read한다
    #[serde(rename = "ref")]
    pub reference: String,
    pub sym: String,
    pub kind: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 점수 기여 내역 — 왜 뽑혔는지 설명한다
    pub why: HashMap<&'static str, f32>,
    pub tokens: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Related {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cochanged: Vec<String>,
    /// 교차 저장소 연결 — grep으로는 원리적으로 나오지 않는 정보
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cross_repo: Vec<CrossRepoHit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossRepoHit {
    pub repo: String,
    pub sym: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub via: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pack {
    pub budget: usize,
    pub used: usize,
    pub seeds: Vec<String>,
    pub items: Vec<PackItem>,
    pub related: Related,
    /// 인덱스가 낡아 신뢰할 수 없는 항목. 틀린 좌표를 자신 있게 주는 것보다
    /// 낡았다고 말하는 편이 항상 낫다 (PLAN.md 3.6절).
    pub stale: Vec<String>,
}

/// 노드 종류별 사전확률.
///
/// 파일은 **컨테이너**이지 답이 아니다. 같은 질의에서 파일 노드와 그 안의 심볼이
/// 경쟁하면 심볼이 이겨야 한다 — 에이전트가 원하는 것은 "이 파일 어딘가"가 아니라
/// "이 함수의 이 줄"이기 때문이다.
fn kind_prior(kind: NodeKind) -> f32 {
    match kind {
        NodeKind::Symbol => 1.0,
        NodeKind::Route => 0.85,
        NodeKind::ApiCall => 0.7,
        NodeKind::File => 0.30,
        _ => 0.2,
    }
}

/// 토큰 수 추정.
///
/// Claude의 토크나이저는 공개되어 있지 않으므로 **추정치**다. 코드에서 경험적으로
/// 문자 3.6개당 1토큰에 가깝다. 예산을 넘지 않게 하는 것이 목적이므로
/// 약간 보수적으로(=많게) 잡는다.
pub fn estimate_tokens(text: &str) -> usize {
    (text.chars().count() as f32 / 3.6).ceil() as usize
}

#[derive(Debug, Clone)]
pub struct PackOptions {
    pub budget: usize,
    pub weights: RankWeights,
    pub seed_limit: usize,
    pub candidate_limit: usize,
    pub damping: f32,
}

impl Default for PackOptions {
    fn default() -> Self {
        PackOptions {
            budget: 4000,
            weights: RankWeights::default(),
            seed_limit: 12,
            candidate_limit: 120,
            damping: DEFAULT_DAMPING,
        }
    }
}

/// 태스크 문장 하나로 컨텍스트 팩을 만든다. PLAN.md 3.5절의 5단계 파이프라인.
pub fn build_pack(
    store: &SqliteStore,
    graph: &MemGraph,
    task: &str,
    repo_roots: &HashMap<String, std::path::PathBuf>,
    opts: &PackOptions,
) -> Result<Pack> {
    // ── 1. 시드: FTS5 BM25 ──
    let hits = store.search(task, opts.seed_limit)?;
    let max_bm25 = hits.first().map(|h| h.score).unwrap_or(1.0).max(1e-6);

    let mut bm25: HashMap<String, f32> = HashMap::new();
    let mut seed_idx = Vec::new();
    let mut seeds = Vec::new();
    for h in &hits {
        bm25.insert(h.node.id.0.clone(), h.score / max_bm25);
        if let Some(i) = graph.index_of(&h.node.id) {
            seed_idx.push(i);
        }
        seeds.push(h.node.name.clone());
    }
    if hits.is_empty() {
        return Ok(Pack {
            budget: opts.budget,
            used: 0,
            seeds,
            items: Vec::new(),
            related: Related::default(),
            stale: Vec::new(),
        });
    }

    // ── 2. 확장: 메모리 인접리스트에서 PPR ──
    let ppr = graph.personalized_pagerank(&seed_idx, opts.damping, 25);
    let central = graph.degree_centrality();
    let max_ppr = ppr.iter().cloned().fold(1e-6f32, f32::max);

    // ── 3~4. 후보 수집 + 랭킹 ──
    let w = &opts.weights;
    let mut scored: Vec<(usize, f32, HashMap<&'static str, f32>)> = Vec::new();
    for i in 0..graph.len() {
        let id = graph.id_at(i);
        let p = ppr[i] / max_ppr;
        let b = bm25.get(&id.0).copied().unwrap_or(0.0);
        if p < 1e-4 && b == 0.0 {
            continue;
        }
        let Some(kind) = store.node_kind(id)? else { continue };
        let prior = kind_prior(kind);
        let c = central[i];
        let score =
            (w.alpha_bm25 * b + w.beta_ppr * p + w.epsilon_central * c) * prior;
        let mut why = HashMap::new();
        if b > 0.0 {
            why.insert("bm25", (b * 100.0).round() / 100.0);
        }
        why.insert("ppr", (p * 100.0).round() / 100.0);
        why.insert("prior", prior);
        scored.push((i, score, why));
    }
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(opts.candidate_limit);

    // ── 5. 예산 렌더링: 점수순 greedy, 예산이 줄면 L2→L1→L0 강등 ──
    let mut items = Vec::new();
    let mut stale = Vec::new();
    let mut used = 0usize;
    let mut full_body_budget = 3usize; // L2는 상위 소수에만
    // 이미 심볼이 담긴 (repo, path) — 같은 파일의 File 노드를 억제한다
    let mut covered_files: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();

    for (rank_pos, (idx, score, why)) in scored.iter().enumerate() {
        if used >= opts.budget {
            break;
        }
        let id = graph.id_at(*idx);
        let Some(node) = store.get_node(id)? else { continue };
        if !matches!(node.kind, NodeKind::Symbol | NodeKind::File | NodeKind::Route) {
            continue;
        }
        // 같은 파일의 심볼이 이미 담겼으면 파일 노드는 중복이다.
        if node.kind == NodeKind::File {
            if let Some(path) = node.path.as_deref() {
                let key = (node.repo.clone(), path.to_string());
                if covered_files.contains(&key) {
                    continue;
                }
            }
        }

        // 지연 검증 — 인덱스가 낡았으면 좌표를 신뢰할 수 없다 (PLAN.md 3.6절)
        let body_source = read_verified(&node, repo_roots);
        if matches!(body_source, Verified::Stale) {
            stale.push(node.reference().unwrap_or_else(|| node.id.0.clone()));
            continue;
        }

        let mut tier = if rank_pos < full_body_budget && matches!(body_source, Verified::Fresh(_)) {
            Tier::L2
        } else if *score > 0.15 {
            Tier::L1
        } else {
            Tier::L0
        };

        // 예산에 맞을 때까지 강등한다.
        let item = loop {
            let candidate = render(&node, tier, &body_source, why.clone(), *score);
            if used + candidate.tokens <= opts.budget {
                break Some(candidate);
            }
            match tier.lower() {
                Some(t) => tier = t,
                None => break None,
            }
        };

        let Some(item) = item else { continue };
        if tier == Tier::L2 {
            full_body_budget = full_body_budget.saturating_sub(1);
        }
        if node.kind == NodeKind::Symbol {
            if let Some(path) = node.path.as_deref() {
                covered_files.insert((node.repo.clone(), path.to_string()));
            }
        }
        used += item.tokens;
        items.push(item);
    }

    // 교차 저장소 연결 — 이 프로젝트의 존재 이유 (PLAN.md 3.9절)
    let related = collect_related(store, graph, &seed_idx, &items)?;

    Ok(Pack {
        budget: opts.budget,
        used,
        seeds,
        items,
        related,
        stale,
    })
}

enum Verified {
    /// 파일을 읽었고 해시가 일치한다
    Fresh(String),
    /// 파일이 없거나 읽을 수 없다 — 본문 없이 좌표만 준다
    Unknown,
    /// 해시 불일치 — 인덱스가 낡았다
    Stale,
}

fn read_verified(node: &Node, roots: &HashMap<String, std::path::PathBuf>) -> Verified {
    let (Some(rel), Some(root)) = (node.path.as_deref(), roots.get(&node.repo)) else {
        return Verified::Unknown;
    };
    let abs = npath::to_extended_length(&root.join(rel));
    let Ok(bytes) = std::fs::read(&abs) else {
        return Verified::Unknown;
    };
    if let Some(expected) = node.content_hash.as_deref() {
        if npath::content_hash(&bytes) != expected {
            return Verified::Stale;
        }
    }
    match String::from_utf8(bytes) {
        Ok(text) => Verified::Fresh(text),
        Err(_) => Verified::Unknown,
    }
}

fn slice_lines(text: &str, start: u32, end: u32, cap: usize) -> String {
    text.lines()
        .skip(start.saturating_sub(1) as usize)
        .take(((end.saturating_sub(start) + 1) as usize).min(cap))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render(
    node: &Node,
    tier: Tier,
    source: &Verified,
    why: HashMap<&'static str, f32>,
    _score: f32,
) -> PackItem {
    let reference = node.reference().unwrap_or_else(|| node.id.0.clone());
    let kind = node
        .attrs
        .get("symbol_kind")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| node.kind.as_str())
        .to_string();

    let (doc, body) = match tier {
        Tier::L0 => (None, None),
        Tier::L1 => {
            let body = match (source, node.span) {
                (Verified::Fresh(text), Some(span)) => {
                    Some(slice_lines(text, span.start_line, span.end_line, 15))
                }
                _ => None,
            };
            (node.doc.clone(), body)
        }
        Tier::L2 => {
            let body = match (source, node.span) {
                (Verified::Fresh(text), Some(span)) => {
                    Some(slice_lines(text, span.start_line, span.end_line, 400))
                }
                _ => None,
            };
            (node.doc.clone(), body)
        }
    };

    let mut item = PackItem {
        tier: tier.as_str(),
        reference,
        sym: node.name.clone(),
        kind,
        repo: node.repo.clone(),
        sig: node.signature.clone(),
        doc,
        body,
        why,
        tokens: 0,
    };
    item.tokens = estimate_tokens(&serde_json::to_string(&item).unwrap_or_default());
    item
}

fn collect_related(
    store: &SqliteStore,
    graph: &MemGraph,
    seeds: &[usize],
    items: &[PackItem],
) -> Result<Related> {
    let mut related = Related::default();
    let mut seen = std::collections::HashSet::new();
    let item_repos: std::collections::HashSet<&str> =
        items.iter().map(|i| i.repo.as_str()).collect();

    // 팩에 담긴 심볼에서 CALLS_API / HANDLES 를 양방향으로 한 홉 편다.
    let mut frontier: Vec<usize> = seeds.to_vec();
    for item in items.iter().take(20) {
        // 아이템의 노드 인덱스를 찾기 위해 좌표로 되짚는다.
        if let Some(i) = graph.index_of(&NodeId(format!("sym:{}/{}", item.repo, item.reference))) {
            frontier.push(i);
        }
    }

    for &i in &frontier {
        for kind in [EdgeKind::CallsApi, EdgeKind::Handles] {
            for n in graph.neighbors_of_kind(i, kind) {
                let id = graph.id_at(n).clone();
                // 라우트를 거쳐 반대편으로 한 홉 더
                for m in graph
                    .neighbors_of_kind(n, EdgeKind::Handles)
                    .into_iter()
                    .chain(std::iter::once(n))
                {
                    let Some(node) = store.get_node(graph.id_at(m))? else { continue };
                    if item_repos.contains(node.repo.as_str()) && node.kind != crate::model::NodeKind::Route {
                        continue;
                    }
                    if !seen.insert(node.id.0.clone()) {
                        continue;
                    }
                    related.cross_repo.push(CrossRepoHit {
                        repo: node.repo.clone(),
                        sym: node.name.clone(),
                        reference: node.reference().unwrap_or_default(),
                        via: kind.as_str().to_uppercase(),
                    });
                }
                let _ = id;
            }
        }
    }
    related.cross_repo.truncate(8);
    Ok(related)
}

/// 저장소 이름 → 루트 경로. 지연 검증에서 실제 파일을 읽는 데 필요하다.
pub fn repo_roots(config: &crate::Config) -> HashMap<String, std::path::PathBuf> {
    config
        .solution
        .repos
        .iter()
        .filter_map(|p| {
            let canonical = p.canonicalize().ok()?;
            let name = canonical.file_name()?.to_string_lossy().to_string();
            Some((name, canonical))
        })
        .collect()
}

/// 사람이 읽는 형태로 렌더링. TUI 팩 미리보기와 CLI가 공유한다.
pub fn render_text(pack: &Pack) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "budget {} · used {} ({}%)\nseeds: {}\n\n",
        pack.budget,
        pack.used,
        if pack.budget > 0 { pack.used * 100 / pack.budget } else { 0 },
        pack.seeds.join(", ")
    ));
    out.push_str(&format!("{:<5}{:>7}  {:<28} {}\n", "tier", "tok", "symbol", "ref"));
    for i in &pack.items {
        out.push_str(&format!(
            "{:<5}{:>7}  {:<28} {}\n",
            i.tier, i.tokens, truncate(&i.sym, 28), i.reference
        ));
    }
    if !pack.related.cross_repo.is_empty() {
        out.push_str("\n교차 저장소\n");
        for c in &pack.related.cross_repo {
            out.push_str(&format!("  ✦ [{}] {} — {} ({})\n", c.repo, c.sym, c.reference, c.via));
        }
    }
    if !pack.stale.is_empty() {
        out.push_str("\n⚠ 인덱스가 낡은 항목 (직접 Read 권장)\n");
        for s in &pack.stale {
            out.push_str(&format!("  {s}\n"));
        }
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).chain(std::iter::once('…')).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_estimate_is_conservative() {
        // 예산 초과를 막는 것이 목적이므로 과소평가하면 안 된다.
        let text = "pub fn find_order(id: u32) -> Option<Order> { lookup(id) }";
        let est = estimate_tokens(text);
        assert!(est >= text.len() / 5, "너무 적게 추정: {est}");
        assert!(est <= text.len(), "너무 많이 추정: {est}");
    }

    #[test]
    fn tiers_demote_downward() {
        assert_eq!(Tier::L2.lower(), Some(Tier::L1));
        assert_eq!(Tier::L1.lower(), Some(Tier::L0));
        assert_eq!(Tier::L0.lower(), None);
    }

    #[test]
    fn slice_respects_cap() {
        let text = (1..=100).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
        let s = slice_lines(&text, 10, 90, 5);
        assert_eq!(s.lines().count(), 5);
        assert_eq!(s.lines().next().unwrap(), "10");
    }
}
