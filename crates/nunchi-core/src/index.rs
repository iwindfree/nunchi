//! 인덱싱 — 파일 워크 + tree-sitter 추출 + 이름 기반 해소
//!
//! 2패스다. 1패스에서 파일·심볼을 모두 만들어야 2패스에서 호출을 해소할 수 있다
//! (앞 파일이 뒤 파일의 심볼을 호출할 수 있으므로).

use crate::config::Config;
use crate::extract::{self, SupportedLang};
use crate::lang;
use crate::model::*;
use crate::path as npath;
use crate::resolve::{ResolveStats, SymbolTable, UnresolvedTally};
use crate::store::{sqlite::SqliteStore, Store};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeMap;
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
}

pub fn index_all(config: &Config, store: &mut SqliteStore) -> Result<IndexStats> {
    let excludes = build_exclude_set(&config.index.exclude)?;
    let mut stats = IndexStats::default();
    let mut table = SymbolTable::default();
    let mut tally = UnresolvedTally::default();
    let mut pending: Vec<PendingFile> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    // ---- 1패스: 파일·심볼 노드 생성 ----
    for root in &config.solution.repos {
        let root = root
            .canonicalize()
            .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", root.display()))?;
        let repo = repo_name(&root);
        scan_repo(
            &repo, &root, config, &excludes, store, &mut stats, &mut table, &mut pending,
            &mut nodes, &mut edges,
        )?;
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

    stats.top_unresolved = tally.top(8);
    stats.nodes = store.upsert_nodes(&nodes)?;
    stats.edges = store.upsert_edges(&edges)?;
    persist_metrics(store, &stats)?;
    Ok(stats)
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
    store: &mut SqliteStore,
    stats: &mut IndexStats,
    table: &mut SymbolTable,
    pending: &mut Vec<PendingFile>,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) -> Result<()> {
    let (branch, head) = git_head(root);
    store.record_repo(repo, &npath::normalize(root), branch.as_deref(), head.as_deref())?;

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
        nodes.push(file_node);
        edges.push(Edge::new(
            repo_id.clone(),
            file_id.clone(),
            EdgeKind::Contains,
            Provenance::Fast,
        ));
        table.insert_file(&rel, file_id.clone());
        stats.files_indexed += 1;

        let counter = stats.by_lang.entry(language.to_string()).or_insert((0, 0));
        counter.0 += 1;

        // 파서가 없는 언어(yaml/json 등)는 파일 노드까지만.
        let Some(sl) = SupportedLang::from_name(language) else { continue };
        let facts = match extract::extract(sl, abs, source) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("추출 실패 {rel}: {e}");
                continue;
            }
        };
        if !facts.had_parse_error {
            counter.1 += 1;
        }

        let mut symbol_spans = Vec::new();
        for sym in &facts.symbols {
            let sym_id = NodeId::symbol(repo, &rel, &sym.name);
            let mut node = Node::new(sym_id.clone(), NodeKind::Symbol, &sym.name, repo);
            node.path = Some(rel.clone());
            node.span = Some(sym.span);
            node.signature = sym.signature.clone();
            node.doc = sym.doc.clone();
            node.lang = Some(language.to_string());
            node.attrs = serde_json::json!({ "symbol_kind": sym.kind });
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

        pending.push(PendingFile {
            repo: repo.to_string(),
            rel,
            lang: language.to_string(),
            file_id,
            facts,
            symbol_spans,
        });
    }
    Ok(())
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
