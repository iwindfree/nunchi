//! 벤치 하네스 — 절감을 주장하려면 재야 한다 (PLAN.md Phase 0)
//!
//! # 이것이 재는 것과 재지 않는 것
//!
//! **재지 않는다**: 실제 에이전트 세션. 그건 모델 호출이 필요하고 비결정적이다.
//!
//! **잰다**: 같은 태스크에 대해
//! - `grounded` — `nunchi pack`이 만든 컨텍스트의 토큰 수
//! - `ungrounded` — 에이전트가 grep 후 매칭 파일을 통째로 읽었을 때의 토큰 수
//! - `recall` — 정답 좌표가 팩 안에 들어 있는가
//!
//! ungrounded는 **대리 지표**다. 실제 에이전트는 더 똑똑하게 읽을 수도, 더
//! 헤맬 수도 있다. 다만 "구조를 모르는 상태에서 이름이 걸리는 파일을 읽는다"는
//! 행동은 실제 관측과 부합하며, 두 값을 같은 태스크·같은 인덱스로 재므로
//! **상대 비교**는 유효하다. 절대 절감률로 인용하면 안 된다.

use crate::graph::MemGraph;
use crate::pack::{self, estimate_tokens, PackOptions};
use crate::store::{sqlite::SqliteStore, Store};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// `bench/tasks.jsonl` 한 줄.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchTask {
    /// 에이전트에게 줄 법한 한 문장
    pub task: String,
    /// 이 태스크를 풀려면 반드시 봐야 하는 좌표들 (`path` 또는 `path:line`).
    /// 부분 경로 일치로 판정하므로 `OrderService.java` 처럼 짧게 써도 된다.
    #[serde(default)]
    pub expect: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskResult {
    pub task: String,
    pub grounded_tokens: usize,
    pub ungrounded_tokens: usize,
    pub ungrounded_files: usize,
    pub saving_pct: f32,
    /// 정답 좌표 중 팩에 담긴 비율. 토큰만 줄고 답을 놓치면 무의미하다.
    pub recall: Option<f32>,
    pub missed: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchReport {
    pub tasks: Vec<TaskResult>,
    pub mean_grounded: usize,
    pub mean_ungrounded: usize,
    pub mean_saving_pct: f32,
    pub mean_recall: Option<f32>,
    pub note: String,
}

pub fn load_tasks(path: &Path) -> Result<Vec<BenchTask>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("태스크 파일을 읽을 수 없습니다: {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        out.push(
            serde_json::from_str(line)
                .with_context(|| format!("{}:{} 파싱 실패", path.display(), i + 1))?,
        );
    }
    Ok(out)
}

pub fn run(
    store: &SqliteStore,
    graph: &MemGraph,
    roots: &HashMap<String, PathBuf>,
    tasks: &[BenchTask],
    opts: &PackOptions,
) -> Result<BenchReport> {
    let mut results = Vec::new();

    for task in tasks {
        let pack = pack::build_pack(store, graph, &task.task, roots, opts)?;

        // ungrounded 대리: 질의어가 걸리는 파일들을 통째로 읽었다고 본다.
        let expanded = opts.synonyms.expand_query(&task.task);
        let hits = store.search(&expanded, 40)?;
        let mut files: HashMap<(String, String), usize> = HashMap::new();
        for h in &hits {
            let (Some(path), Some(root)) = (h.node.path.as_deref(), roots.get(&h.node.repo)) else {
                continue;
            };
            let key = (h.node.repo.clone(), path.to_string());
            if files.contains_key(&key) {
                continue;
            }
            let bytes = std::fs::read(crate::path::to_extended_length(&root.join(path)))
                .map(|b| b.len())
                .unwrap_or(0);
            // 파일 전체를 컨텍스트에 넣었을 때의 비용
            files.insert(key, estimate_tokens(&"x".repeat(bytes)));
        }
        let ungrounded_tokens: usize = files.values().sum();
        let grounded_tokens = pack.used;

        let saving_pct = if ungrounded_tokens > 0 {
            (1.0 - grounded_tokens as f32 / ungrounded_tokens as f32) * 100.0
        } else {
            0.0
        };

        // recall — 정답 좌표가 팩 안에 있는가
        let (recall, missed) = if task.expect.is_empty() {
            (None, Vec::new())
        } else {
            let refs: Vec<&str> = pack.items.iter().map(|i| i.reference.as_str()).collect();
            let missed: Vec<String> = task
                .expect
                .iter()
                .filter(|want| !refs.iter().any(|got| got.contains(want.as_str())))
                .cloned()
                .collect();
            let hit = task.expect.len() - missed.len();
            (Some(hit as f32 / task.expect.len() as f32), missed)
        };

        results.push(TaskResult {
            task: task.task.clone(),
            grounded_tokens,
            ungrounded_tokens,
            ungrounded_files: files.len(),
            saving_pct,
            recall,
            missed,
        });
    }

    let n = results.len().max(1);
    let recalls: Vec<f32> = results.iter().filter_map(|r| r.recall).collect();

    Ok(BenchReport {
        mean_grounded: results.iter().map(|r| r.grounded_tokens).sum::<usize>() / n,
        mean_ungrounded: results.iter().map(|r| r.ungrounded_tokens).sum::<usize>() / n,
        mean_saving_pct: results.iter().map(|r| r.saving_pct).sum::<f32>() / n as f32,
        mean_recall: (!recalls.is_empty())
            .then(|| recalls.iter().sum::<f32>() / recalls.len() as f32),
        note: "ungrounded는 대리 지표다(질의어가 걸리는 파일을 통째로 읽었다고 가정). \
               실제 에이전트 세션이 아니므로 절대 절감률로 인용하지 말 것."
            .into(),
        tasks: results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jsonl_and_skips_comments() -> Result<()> {
        let dir = std::env::temp_dir().join("nunchi-bench-test");
        std::fs::create_dir_all(&dir)?;
        let p = dir.join("tasks.jsonl");
        std::fs::write(
            &p,
            "# 주석은 건너뛴다\n\
             {\"task\":\"댓글 삭제\",\"expect\":[\"CommentController.java\"]}\n\
             \n\
             {\"task\":\"주문 조회\"}\n",
        )?;
        let tasks = load_tasks(&p)?;
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].expect, vec!["CommentController.java"]);
        assert!(tasks[1].expect.is_empty(), "expect는 선택 항목이다");
        Ok(())
    }
}
