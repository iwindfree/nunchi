//! git 이력 — 동시변경 결합도 (PLAN.md 3절 Phase 3)
//!
//! 이 계층은 **브랜치와 무관하다.** git 이력에 기반하므로 checkout해도 바뀌지 않고
//! (PLAN.md 3.7절), 커밋 시점에만 갱신하면 된다(PLAN.md 3.6절 C 계층).
//!
//! 동시변경은 구조 그래프가 못 보는 관계를 잡는다. 호출도 import도 없지만
//! 늘 함께 바뀌는 두 파일은 실제로 결합되어 있다.

use crate::model::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct HistoryStats {
    pub commits: usize,
    pub authors: usize,
    pub cochange_pairs: usize,
}

/// 한 커밋에서 함께 바뀐 파일이 이보다 많으면 결합도 신호로 쓰지 않는다.
/// 대규모 리팩터링·포맷팅 커밋이 모든 파일을 서로 묶어버리는 것을 막는다.
const MAX_FILES_PER_COMMIT: usize = 25;

/// 동시변경 엣지를 만들 최소 동반 횟수.
const MIN_COCHANGE: usize = 2;

pub fn index_history(
    repo: &str,
    root: &Path,
    max_commits: usize,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) -> Result<HistoryStats> {
    let mut stats = HistoryStats::default();

    // %x00 구분자로 커밋 헤더와 파일 목록을 한 번에 받는다.
    let out = Command::new("git")
        .args([
            "log",
            &format!("-n{max_commits}"),
            "--name-only",
            "--no-merges",
            "--pretty=format:%x00%H%x1f%an%x1f%ae%x1f%at%x1f%s",
        ])
        .current_dir(root)
        .output();

    let Ok(out) = out else { return Ok(stats) };
    if !out.status.success() {
        return Ok(stats);
    }
    let text = String::from_utf8_lossy(&out.stdout);

    let mut authors: HashMap<String, NodeId> = HashMap::new();
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

    for record in text.split('\u{0}').filter(|r| !r.trim().is_empty()) {
        let mut lines = record.lines();
        let Some(header) = lines.next() else { continue };
        let parts: Vec<&str> = header.split('\u{1f}').collect();
        if parts.len() < 5 {
            continue;
        }
        let (sha, author_name, author_email, epoch, subject) =
            (parts[0], parts[1], parts[2], parts[3], parts[4]);

        let files: Vec<String> = lines
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.replace('\\', "/"))
            .collect();
        if files.is_empty() {
            continue;
        }

        let commit_id = NodeId(format!("commit:{repo}/{sha}"));
        let mut commit = Node::new(commit_id.clone(), NodeKind::Commit, &sha[..7.min(sha.len())], repo);
        commit.doc = Some(subject.to_string());
        commit.attrs = serde_json::json!({ "sha": sha, "epoch": epoch.parse::<i64>().unwrap_or(0) });
        nodes.push(commit);
        stats.commits += 1;

        let author_id = authors
            .entry(author_email.to_string())
            .or_insert_with(|| {
                let id = NodeId(format!("author:{author_email}"));
                let mut n = Node::new(id.clone(), NodeKind::Author, author_name, repo);
                n.attrs = serde_json::json!({ "email": author_email });
                nodes.push(n);
                id
            })
            .clone();
        edges.push(Edge::new(
            commit_id.clone(),
            author_id,
            EdgeKind::AuthoredBy,
            Provenance::Precise,
        ));

        for f in &files {
            edges.push(Edge::new(
                NodeId::file(repo, f),
                commit_id.clone(),
                EdgeKind::ModifiedBy,
                Provenance::Precise,
            ));
        }

        // 거대 커밋은 결합도 신호에서 제외한다.
        if files.len() > MAX_FILES_PER_COMMIT {
            continue;
        }
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let (a, b) = if files[i] < files[j] {
                    (files[i].clone(), files[j].clone())
                } else {
                    (files[j].clone(), files[i].clone())
                };
                *pair_counts.entry((a, b)).or_default() += 1;
            }
        }
    }
    stats.authors = authors.len();

    for ((a, b), count) in pair_counts {
        if count < MIN_COCHANGE {
            continue;
        }
        // 동반 횟수가 많을수록 강한 결합. 로그로 눌러 과대평가를 막는다.
        let weight = (count as f32).ln_1p();
        let (ia, ib) = (NodeId::file(repo, &a), NodeId::file(repo, &b));
        edges.push(
            Edge::new(ia.clone(), ib.clone(), EdgeKind::CoChangedWith, Provenance::Precise)
                .with_weight(weight),
        );
        edges.push(
            Edge::new(ib, ia, EdgeKind::CoChangedWith, Provenance::Precise).with_weight(weight),
        );
        stats.cochange_pairs += 1;
    }

    Ok(stats)
}
