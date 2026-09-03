//! 인덱싱 — 파일 워크 + tree-sitter 추출 + 이름 기반 해소
//!
//! 2패스다. 1패스에서 파일·심볼을 모두 만들어야 2패스에서 호출을 해소할 수 있다
//! (앞 파일이 뒤 파일의 심볼을 호출할 수 있으므로).

use crate::config::Config;
use crate::extract::{self, SupportedLang};
use crate::framework::{self, FrameworkFacts};
use crate::lang;
use crate::model::*;
use crate::path as npath;
use crate::resolve::{ResolveStats, SymbolTable, UnresolvedTally};
use crate::store::{sqlite::SqliteStore, Store};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub repos: usize,
    pub files_seen: usize,
    pub files_indexed: usize,
    pub files_skipped_size: usize,
    pub files_skipped_binary: usize,
    pub symbols: usize,
    pub nodes: usize,
    pub edges: usize,
    pub calls: ResolveStats,
    pub imports_internal: usize,
    pub imports_external: usize,
    /// 언어별 (파일 수, 파싱 성공 수). `nunchi doctor` 커버리지 표의 원천.
    pub by_lang: BTreeMap<String, (usize, usize)>,
    /// 미해소 호출 이름 상위 — 외부 API인지 추출기 결함인지 판단하는 근거
    pub top_unresolved: Vec<(String, usize)>,
    // ── 프레임워크 의미론 (Phase 1c) ──
    pub routes: usize,
    pub beans: usize,
    pub injects_resolved: usize,
    pub injects_unresolved: usize,
    pub api_calls: usize,
    /// 프런트 호출이 백엔드 라우트에 연결된 수 — v1의 하이라이트 지표
    pub api_calls_linked: usize,
    pub unlinked_api_paths: Vec<String>,
    /// 정적으로 경로를 알 수 없는 호출 — 연결 실패로 세면 지표가 왜곡된다
    pub api_calls_dynamic: usize,
    // ── git 이력 (Phase 3) ──
    pub commits: usize,
    pub authors: usize,
    pub cochange_pairs: usize,
    // ── 콘텐츠 주소 캐시 (Phase 5) ──
    pub cache_hits: usize,
    pub cache_misses: usize,
    /// 사라진 파일 때문에 인덱스에서 제거된 노드 수
    pub pruned: usize,
}

pub fn build_exclude_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p).with_context(|| format!("잘못된 제외 패턴: {p}"))?);
    }
    Ok(builder.build()?)
}

/// 1패스에서 모아두는 파일별 중간 결과.
struct PendingFile {
    repo: String,
    rel: String,
    lang: String,
    file_id: NodeId,
    facts: extract::FileFacts,
    /// 심볼 span — 호출의 소속 심볼을 찾는 데 쓴다
    symbol_spans: Vec<(Span, NodeId)>,
    fw: FrameworkFacts,
    /// 이 파일에서 만든 ApiCall 노드들 — 2패스에서 라우트에 연결한다
    api_call_ids: Vec<(NodeId, String, String, bool)>,
}

pub fn index_all(config: &Config, store: &mut SqliteStore) -> Result<IndexStats> {
    index_all_with_cache(config, store, None)
}

/// 캐시를 함께 쓰는 인덱싱. 브랜치 전환 시 재파싱을 피한다(PLAN 3.7절).
pub fn index_all_with_cache(
    config: &Config,
    store: &mut SqliteStore,
    mut cache: Option<&mut crate::cache::ExtractCache>,
) -> Result<IndexStats> {
    let excludes = build_exclude_set(&config.index.exclude)?;
    let rules = crate::rules::FrameworkRules::effective(&config.framework);
    let mut stats = IndexStats::default();
    let mut table = SymbolTable::default();
    let mut tally = UnresolvedTally::default();
    let mut pending: Vec<PendingFile> = Vec::new();
    let mut seen_by_repo: Vec<(String, Vec<String>)> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    // ---- 1패스: 파일·심볼 노드 생성 ----
    for root in &config.solution.repos {
        let root = root
            .canonicalize()
            .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", root.display()))?;
        let repo = repo_name(&root);
        let seen = scan_repo(
            &repo, &root, config, &excludes, &rules, store, &mut stats, &mut table, &mut pending,
            &mut nodes, &mut edges, cache.as_deref_mut(),
        )?;
        seen_by_repo.push((repo, seen));
        stats.repos += 1;
    }

    // ---- 2패스: 호출·import 해소 ----
    for file in &pending {
        for call in &file.facts.calls {
            let src = enclosing_symbol(&file.symbol_spans, call.line)
                .unwrap_or_else(|| file.file_id.clone());
            for (dst, confidence) in table.resolve_call(&call.callee, &mut stats.calls, &mut tally) {
                if dst == src {
                    continue; // 자기 호출은 그래프에 도움이 안 된다
                }
                edges.push(
                    Edge::new(src.clone(), dst, EdgeKind::Calls, Provenance::Fast)
                        .with_confidence(confidence),
                );
            }
        }

        for spec in &file.facts.imports {
            match table.resolve_import(&file.lang, &file.rel, spec) {
                Some(target) => {
                    stats.imports_internal += 1;
                    edges.push(Edge::new(
                        file.file_id.clone(),
                        target,
                        EdgeKind::Imports,
                        Provenance::Fast,
                    ));
                }
                None => {
                    stats.imports_external += 1;
                    let dep = external_dep_name(&file.lang, spec);
                    let dep_id = NodeId(format!("dep:{dep}"));
                    nodes.push(Node::new(
                        dep_id.clone(),
                        NodeKind::ExternalDep,
                        dep,
                        &file.repo,
                    ));
                    edges.push(Edge::new(
                        file.file_id.clone(),
                        dep_id,
                        EdgeKind::DependsOn,
                        Provenance::Fast,
                    ));
                }
            }
        }
    }

    // ── 교차 저장소 계약 엣지 — v1의 하이라이트 (PLAN.md 3.9절) ──
    // 라우트는 솔루션 전체에서 유일하므로 저장소가 달라도 매칭된다.
    let mut route_index: HashMap<(String, String), NodeId> = HashMap::new();
    for file in &pending {
        for r in &file.fw.routes {
            route_index.insert(
                (r.method.clone(), r.path.clone()),
                route_id(&r.method, &r.path),
            );
        }
    }

    for file in &pending {
        for (call_id, method, path, dynamic) in &file.api_call_ids {
            if *dynamic {
                continue; // 경로를 정적으로 알 수 없다 — 연결 실패가 아니다
            }
            // 정확히 일치하는 라우트 우선, 없으면 메서드 무관(@RequestMapping) 라우트.
            let hit = route_index
                .get(&(method.clone(), path.clone()))
                .map(|id| (id.clone(), 0.9))
                .or_else(|| {
                    route_index
                        .get(&("ANY".to_string(), path.clone()))
                        .map(|id| (id.clone(), 0.6))
                });
            match hit {
                Some((route, confidence)) => {
                    stats.api_calls_linked += 1;
                    edges.push(
                        Edge::new(call_id.clone(), route, EdgeKind::CallsApi, Provenance::Fast)
                            .with_confidence(confidence),
                    );
                }
                None => {
                    if stats.unlinked_api_paths.len() < 8 {
                        stats.unlinked_api_paths.push(format!("{method} {path}"));
                    }
                }
            }
        }

        // DI 주입 — 타입 이름을 심볼로 해소한다.
        for inject in &file.fw.injects {
            let owner = NodeId::symbol(&file.repo, &file.rel, &inject.owner);
            let candidates = table.candidates(&inject.injected_type);
            if candidates.is_empty() {
                stats.injects_unresolved += 1;
                continue;
            }
            stats.injects_resolved += 1;
            let confidence = 0.9 / candidates.len() as f32;
            for dst in candidates {
                edges.push(
                    Edge::new(owner.clone(), dst.clone(), EdgeKind::Injects, Provenance::Fast)
                        .with_confidence(confidence),
                );
            }
        }
    }

    stats.top_unresolved = tally.top(8);
    stats.nodes = store.upsert_nodes(&nodes)?;
    stats.edges = store.upsert_edges(&edges)?;

    // upsert 뒤에 정리한다 — 먼저 지우면 이번에 다시 만든 노드까지 날아간다.
    for (repo, seen) in &seen_by_repo {
        stats.pruned += store.prune_missing_files(repo, seen)?;
    }
    // 파일이 지워지면 그것만 참조하던 의존성·커밋·저자가 고아가 된다.
    stats.pruned += store.prune_orphans()?;
    persist_metrics(store, &stats)?;
    Ok(stats)
}

/// 라우트 노드 ID. 솔루션 전역에서 유일해야 프런트–백엔드가 같은 노드를 가리킨다.
fn route_id(method: &str, path: &str) -> NodeId {
    NodeId(format!("route:{method} {path}"))
}

#[allow(clippy::too_many_arguments)]
fn extract_framework(
    sl: SupportedLang,
    language: &str,
    rules: &crate::rules::FrameworkRules,
    source: &str,
    abs: &Path,
    repo: &str,
    rel: &str,
    file_id: &NodeId,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    stats: &mut IndexStats,
) -> (FrameworkFacts, Vec<(NodeId, String, String, bool)>) {
    let mut parser = tree_sitter::Parser::new();
    let Ok(()) = parser.set_language(&sl.language_for(abs)) else {
        return (FrameworkFacts::default(), Vec::new());
    };
    let Some(tree) = parser.parse(source, None) else {
        return (FrameworkFacts::default(), Vec::new());
    };
    let bytes = source.as_bytes();
    let root = tree.root_node();

    let mut fw = framework::extract_annotated(root, bytes, language, rules);
    fw.api_calls = framework::extract_api_calls(root, bytes, language, rules);

    for route in &fw.routes {
        let id = route_id(&route.method, &route.path);
        let mut node = Node::new(
            id,
            NodeKind::Route,
            format!("{} {}", route.method, route.path),
            repo,
        );
        node.path = Some(rel.to_string());
        node.span = Some(route.span);
        node.signature = Some(format!("{} {}", route.method, route.raw_path));
        node.lang = Some(language.to_string());
        nodes.push(node);
        stats.routes += 1;
    }
    stats.beans += fw.beans.len();

    let mut api_call_ids = Vec::new();
    for (i, call) in fw.api_calls.iter().enumerate() {
        let id = NodeId(format!("api:{repo}/{rel}#{}:{i}", call.span.start_line));
        let mut node = Node::new(
            id.clone(),
            NodeKind::ApiCall,
            format!("{} {}", call.method, call.path),
            repo,
        );
        node.path = Some(rel.to_string());
        node.span = Some(call.span);
        node.signature = Some(format!("{} {}", call.method, call.raw_path));
        node.lang = Some(language.to_string());
        node.attrs = serde_json::json!({ "dynamic": call.dynamic });
        nodes.push(node);
        edges.push(Edge::new(
            file_id.clone(),
            id.clone(),
            EdgeKind::Contains,
            Provenance::Fast,
        ));
        api_call_ids.push((id, call.method.clone(), call.path.clone(), call.dynamic));
        if call.dynamic {
            stats.api_calls_dynamic += 1;
        } else {
            stats.api_calls += 1;
        }
    }

    (fw, api_call_ids)
}

/// 호출이 속한 심볼 — span이 그 줄을 포함하는 것 중 **가장 좁은** 것.
/// 중첩 정의(클래스 안 메서드)에서 메서드를 고르기 위해서다.
fn enclosing_symbol(spans: &[(Span, NodeId)], line: u32) -> Option<NodeId> {
    spans
        .iter()
        .filter(|(s, _)| s.start_line <= line && line <= s.end_line)
        .min_by_key(|(s, _)| s.end_line - s.start_line)
        .map(|(_, id)| id.clone())
}

fn external_dep_name(lang: &str, spec: &str) -> String {
    match lang {
        // `org.springframework.web...` → `org.springframework`
        "java" => spec.split('.').take(2).collect::<Vec<_>>().join("."),
        // `@scope/pkg/sub` → `@scope/pkg`, `react-dom/client` → `react-dom`
        "typescript" | "javascript" => {
            let parts: Vec<&str> = spec.split('/').collect();
            if spec.starts_with('@') && parts.len() >= 2 {
                format!("{}/{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            }
        }
        _ => spec.to_string(),
    }
}

fn repo_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string())
}

#[allow(clippy::too_many_arguments)]
fn scan_repo(
    repo: &str,
    root: &Path,
    config: &Config,
    excludes: &GlobSet,
    rules: &crate::rules::FrameworkRules,
    store: &mut SqliteStore,
    stats: &mut IndexStats,
    table: &mut SymbolTable,
    pending: &mut Vec<PendingFile>,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    mut cache: Option<&mut crate::cache::ExtractCache>,
) -> Result<Vec<String>> {
    let mut seen_paths: Vec<String> = Vec::new();
    let (branch, head) = git_head(root);
    store.record_repo(repo, &npath::normalize(root), branch.as_deref(), head.as_deref())?;

    // git 이력은 브랜치 무관·커밋 시점 갱신이다(PLAN 3.6·3.7절).
    if config.index.max_commits > 0 {
        let h = crate::history::index_history(repo, root, config.index.max_commits, nodes, edges)?;
        stats.commits += h.commits;
        stats.authors += h.authors;
        stats.cochange_pairs += h.cochange_pairs;
    }

    let repo_id = NodeId::repo(repo);
    nodes.push(Node::new(repo_id.clone(), NodeKind::Repo, repo, repo));

    // filter_entry로 **디렉터리 자체를 쳐낸다.** 파일 단위로만 걸러내면
    // node_modules/·target/·build/ 안까지 전부 걸어 들어간다.
    let prune_root = root.to_path_buf();
    let prune_set = excludes.clone();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(false)
        .filter_entry(move |entry| {
            let Some(rel) = npath::relative_to(&prune_root, entry.path()) else {
                return true;
            };
            if rel.is_empty() {
                return true;
            }
            if entry.file_type().is_some_and(|t| t.is_dir()) {
                !(prune_set.is_match(&rel) || prune_set.is_match(format!("{rel}/")))
            } else {
                !prune_set.is_match(&rel)
            }
        })
        .build();

    for entry in walker {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let abs = entry.path();
        let Some(rel) = npath::relative_to(root, abs) else { continue };
        stats.files_seen += 1;

        if excludes.is_match(&rel) {
            continue;
        }
        let Some(language) = lang::detect(abs) else { continue };
        let Ok(meta) = entry.metadata() else { continue };
        if meta.len() > config.index.max_file_bytes {
            stats.files_skipped_size += 1;
            continue;
        }
        let Ok(bytes) = std::fs::read(npath::to_extended_length(abs)) else { continue };
        let Ok(source) = std::str::from_utf8(&bytes) else {
            stats.files_skipped_binary += 1;
            continue;
        };

        let line_count = bytes.iter().filter(|b| **b == b'\n').count() as u32 + 1;
        let file_id = NodeId::file(repo, &rel);
        let mut file_node = Node::new(file_id.clone(), NodeKind::File, &rel, repo);
        file_node.path = Some(rel.clone());
        file_node.lang = Some(language.to_string());
        file_node.content_hash = Some(npath::content_hash(&bytes));
        file_node.span = Some(Span { start_line: 1, end_line: line_count });
        // 랭킹의 recency 항이 쓴다. 최근 바뀐 코드가 대개 지금 관심사다.
        let file_mtime = meta
            .modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        file_node.mtime = file_mtime;
        nodes.push(file_node);
        edges.push(Edge::new(
            repo_id.clone(),
            file_id.clone(),
            EdgeKind::Contains,
            Provenance::Fast,
        ));
        table.insert_file(&rel, file_id.clone());
        seen_paths.push(rel.clone());
        stats.files_indexed += 1;

        let counter = stats.by_lang.entry(language.to_string()).or_insert((0, 0));
        counter.0 += 1;

        // 파서가 없는 언어(yaml/json 등)는 파일 노드까지만.
        let Some(sl) = SupportedLang::from_name(language) else { continue };
        let hash = npath::content_hash(&bytes);

        // 캐시 조회 — 내용이 같으면 브랜치가 달라도 재파싱하지 않는다.
        let cached = cache
            .as_deref_mut()
            .and_then(|c| c.get(&hash, language))
            .and_then(|p| serde_json::from_str::<extract::FileFacts>(&p).ok());

        let facts = match cached {
            Some(f) => {
                stats.cache_hits += 1;
                f
            }
            None => {
                stats.cache_misses += 1;
                match extract::extract(sl, abs, source) {
                    Ok(f) => {
                        if let (Some(c), Ok(payload)) =
                            (cache.as_deref_mut(), serde_json::to_string(&f))
                        {
                            let _ = c.put(&hash, language, &payload);
                        }
                        f
                    }
                    Err(e) => {
                        tracing::warn!("추출 실패 {rel}: {e}");
                        continue;
                    }
                }
            }
        };
        if !facts.had_parse_error {
            counter.1 += 1;
        }

        // ── 프레임워크 의미론 (Phase 1c) ──
        let (fw, api_call_ids) = extract_framework(
            sl, language, rules, source, abs, repo, &rel, &file_id, nodes, edges, stats,
        );

        // Bean 스테레오타입은 별도 노드를 만들지 않고 클래스 심볼의 속성으로 붙인다.
        // 별도 노드를 만들면 하나의 클래스가 두 개의 정체성을 갖게 된다.
        let stereotypes: HashMap<&str, &str> = fw
            .beans
            .iter()
            .map(|b| (b.name.as_str(), b.stereotype.as_str()))
            .collect();

        let mut symbol_spans = Vec::new();
        for sym in &facts.symbols {
            // partial 타입은 파일이 달라도 같은 노드여야 한다 (C# WinForms Designer).
            let sym_id = if sym.partial {
                NodeId::partial_symbol(repo, &sym.name)
            } else {
                NodeId::symbol(repo, &rel, &sym.name)
            };
            let mut node = Node::new(sym_id.clone(), NodeKind::Symbol, &sym.name, repo);
            node.path = Some(rel.clone());
            node.span = Some(sym.span);
            node.signature = sym.signature.clone();
            node.doc = sym.doc.clone();
            node.lang = Some(language.to_string());
            node.mtime = file_mtime;
            node.attrs = match stereotypes.get(sym.name.as_str()) {
                Some(stereotype) => serde_json::json!({
                    "symbol_kind": sym.kind,
                    "spring_stereotype": stereotype,
                }),
                None => serde_json::json!({ "symbol_kind": sym.kind }),
            };
            nodes.push(node);

            edges.push(Edge::new(
                file_id.clone(),
                sym_id.clone(),
                EdgeKind::Contains,
                Provenance::Fast,
            ));
            edges.push(Edge::new(
                sym_id.clone(),
                file_id.clone(),
                EdgeKind::DefinedIn,
                Provenance::Fast,
            ));
            table.insert_symbol(&sym.name, sym_id.clone());
            symbol_spans.push((sym.span, sym_id));
            stats.symbols += 1;
        }

        for route in &fw.routes {
            edges.push(
                Edge::new(
                    route_id(&route.method, &route.path),
                    NodeId::symbol(repo, &rel, &route.handler),
                    EdgeKind::Handles,
                    Provenance::Fast,
                )
                .with_confidence(0.9),
            );
        }

        pending.push(PendingFile {
            repo: repo.to_string(),
            rel,
            lang: language.to_string(),
            file_id,
            facts,
            symbol_spans,
            fw,
            api_call_ids,
        });
    }
    Ok(seen_paths)
}

/// 해소 지표를 인덱스에 남긴다. `nunchi doctor`가 재계산 없이 읽는다.
fn persist_metrics(store: &mut SqliteStore, stats: &IndexStats) -> Result<()> {
    let metrics = serde_json::json!({
        "calls_total": stats.calls.total,
        "calls_resolved": stats.calls.resolved,
        "calls_ambiguous": stats.calls.ambiguous,
        "calls_unresolved": stats.calls.unresolved,
        "calls_dropped": stats.calls.dropped,
        "call_link_rate": stats.calls.rate(),
        "top_unresolved": stats.top_unresolved.iter()
            .map(|(name, n)| serde_json::json!({"name": name, "count": n}))
            .collect::<Vec<_>>(),
        "routes": stats.routes,
        "beans": stats.beans,
        "api_calls": stats.api_calls,
        "api_calls_dynamic": stats.api_calls_dynamic,
        "api_calls_linked": stats.api_calls_linked,
        "unlinked_api_paths": stats.unlinked_api_paths,
        "injects_resolved": stats.injects_resolved,
        "injects_unresolved": stats.injects_unresolved,
        "pruned": stats.pruned,
        "cache_hits": stats.cache_hits,
        "cache_misses": stats.cache_misses,
        "commits": stats.commits,
        "authors": stats.authors,
        "cochange_pairs": stats.cochange_pairs,
        "imports_internal": stats.imports_internal,
        "imports_external": stats.imports_external,
        "by_lang": stats.by_lang.iter()
            .map(|(l, (files, parsed))| serde_json::json!({
                "lang": l, "files": files, "parsed": parsed
            }))
            .collect::<Vec<_>>(),
    });
    store.set_meta("metrics", &serde_json::to_string(&metrics)?)?;
    Ok(())
}

/// best-effort. git이 없거나 저장소가 아니면 `(None, None)`.
fn git_head(root: &Path) -> (Option<String>, Option<String>) {
    let run = |args: &[&str]| -> Option<String> {
        let out = Command::new("git").args(args).current_dir(root).output().ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
        (!s.is_empty()).then_some(s)
    };
    (run(&["rev-parse", "--abbrev-ref", "HEAD"]), run(&["rev-parse", "HEAD"]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn innermost_symbol_wins() {
        let outer = (Span { start_line: 1, end_line: 100 }, NodeId("class".into()));
        let inner = (Span { start_line: 10, end_line: 20 }, NodeId("method".into()));
        let spans = vec![outer, inner];
        assert_eq!(enclosing_symbol(&spans, 15).unwrap().0, "method");
        assert_eq!(enclosing_symbol(&spans, 50).unwrap().0, "class");
        assert!(enclosing_symbol(&spans, 200).is_none());
    }

    #[test]
    fn external_dep_names_are_grouped() {
        assert_eq!(external_dep_name("java", "org.springframework.web.bind.X"), "org.springframework");
        assert_eq!(external_dep_name("typescript", "react-dom/client"), "react-dom");
        assert_eq!(external_dep_name("typescript", "@tanstack/react-query/build"), "@tanstack/react-query");
    }
}
