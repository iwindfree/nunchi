//! 인덱싱 — 파일 워크 + 내용 해시 (PLAN.md 3.6절 A 계층)
//!
//! 현재는 File 노드까지만 만든다. 심볼 추출(tree-sitter)은 Phase 1의 다음 단계다.

use crate::config::Config;
use crate::lang;
use crate::model::*;
use crate::path as npath;
use crate::store::{sqlite::SqliteStore, Store};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub repos: usize,
    pub files_seen: usize,
    pub files_indexed: usize,
    pub files_skipped_size: usize,
    pub files_skipped_binary: usize,
    pub nodes: usize,
    pub edges: usize,
}

pub fn build_exclude_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p).with_context(|| format!("잘못된 제외 패턴: {p}"))?);
    }
    Ok(builder.build()?)
}

/// 설정의 모든 저장소를 인덱싱한다.
pub fn index_all(config: &Config, store: &mut SqliteStore) -> Result<IndexStats> {
    let excludes = build_exclude_set(&config.index.exclude)?;
    let mut stats = IndexStats::default();

    for root in &config.solution.repos {
        let root = root
            .canonicalize()
            .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", root.display()))?;
        let repo = repo_name(&root);
        index_repo(&repo, &root, config, &excludes, store, &mut stats)?;
        stats.repos += 1;
    }
    Ok(stats)
}

fn repo_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string())
}

fn index_repo(
    repo: &str,
    root: &Path,
    config: &Config,
    excludes: &GlobSet,
    store: &mut SqliteStore,
    stats: &mut IndexStats,
) -> Result<()> {
    let (branch, head) = git_head(root);
    store.record_repo(repo, &npath::normalize(root), branch.as_deref(), head.as_deref())?;

    let repo_id = NodeId::repo(repo);
    let mut nodes = vec![Node::new(repo_id.clone(), NodeKind::Repo, repo, repo)];
    let mut edges = Vec::new();

    // ignore::WalkBuilder는 .gitignore를 존중한다. 그 위에 명시적 제외 패턴을 얹는다.
    //
    // filter_entry로 **디렉터리 자체를 쳐내는 것이 중요하다.** 파일 단위로만 걸러내면
    // node_modules/·target/·build/ 안까지 전부 걸어 들어가 콜드 인덱싱이 느려진다.
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
            // 디렉터리는 경로 자체와 `dir/**` 양쪽으로 판정한다.
            if entry.file_type().is_some_and(|t| t.is_dir()) {
                !(prune_set.is_match(&rel) || prune_set.is_match(format!("{rel}/")))
            } else {
                !prune_set.is_match(&rel)
            }
        })
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let abs = entry.path();
        let Some(rel) = npath::relative_to(root, abs) else {
            continue;
        };
        stats.files_seen += 1;

        if excludes.is_match(&rel) {
            continue;
        }
        let Some(language) = lang::detect(abs) else {
            continue;
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() > config.index.max_file_bytes {
            stats.files_skipped_size += 1;
            continue;
        }

        let bytes = match std::fs::read(npath::to_extended_length(abs)) {
            Ok(b) => b,
            Err(_) => continue,
        };
        // 텍스트가 아니면 파싱 대상이 아니다.
        if std::str::from_utf8(&bytes).is_err() {
            stats.files_skipped_binary += 1;
            continue;
        }

        let line_count = bytes.iter().filter(|b| **b == b'\n').count() as u32 + 1;
        let id = NodeId::file(repo, &rel);
        let mut node = Node::new(id.clone(), NodeKind::File, &rel, repo);
        node.path = Some(rel.clone());
        node.lang = Some(language.to_string());
        node.content_hash = Some(npath::content_hash(&bytes));
        node.span = Some(Span { start_line: 1, end_line: line_count });

        nodes.push(node);
        edges.push(Edge::new(repo_id.clone(), id, EdgeKind::Contains, Provenance::Fast));
        stats.files_indexed += 1;
    }

    stats.nodes += store.upsert_nodes(&nodes)?;
    stats.edges += store.upsert_edges(&edges)?;
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
    (
        run(&["rev-parse", "--abbrev-ref", "HEAD"]),
        run(&["rev-parse", "HEAD"]),
    )
}
