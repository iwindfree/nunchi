//! 탐색 화면과 팩 화면이 쓰는 조회 기능.
//!
//! TUI가 부르던 함수를 그대로 부른다. 화면만 다르고 결과는 같다. 여기에
//! 고유 로직을 두면 CLI·MCP·앱이 서로 다른 답을 내게 되므로 넘기고 받는
//! 일만 한다.

use anyhow::{Context, Result};
use nunchi_core::config::RankWeights;
use nunchi_core::graph::MemGraph;
use nunchi_core::model::{Direction, EdgeKind, Node};
use nunchi_core::store::{Store, sqlite::SqliteStore};
use nunchi_core::{Config, NodeId, pack};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 목록에 한 줄로 보여 줄 심볼.
#[derive(Serialize)]
pub struct Hit {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub repo: String,
    /// `path:line` 좌표. 에이전트에게 넘기는 값과 같은 것이다.
    pub reference: Option<String>,
    pub signature: Option<String>,
    pub score: f32,
}

fn hit(node: &Node, score: f32) -> Hit {
    Hit {
        id: node.id.0.clone(),
        name: node.name.clone(),
        kind: node.kind.as_str().to_string(),
        repo: node.repo.clone(),
        reference: node.reference(),
        signature: node.signature.clone(),
        score,
    }
}

#[derive(Serialize)]
pub struct PackView {
    pub pack: pack::Pack,
    /// 에이전트가 실제로 받는 형태. 그대로 복사해 쓸 수 있다.
    pub text: String,
}

/// 열어 둔 인덱스.
///
/// 팩을 만들 때마다 그래프를 다시 읽으면 느리므로 한 번 읽어 두고 다시 쓴다.
/// 인덱싱을 새로 하거나 설정을 고치면 버리고 다시 연다.
pub struct Session {
    pub config_path: PathBuf,
    config: Config,
    store: SqliteStore,
    graph: MemGraph,
    roots: HashMap<String, PathBuf>,
}

impl Session {
    pub fn open(config_path: &Path) -> Result<Session> {
        let config = Config::load(config_path)?;
        let db = config_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(".nunchi")
            .join("graph.db");
        if !db.is_file() {
            anyhow::bail!("아직 인덱싱하지 않았습니다. 인덱싱 화면에서 먼저 실행하십시오.");
        }
        let store = SqliteStore::open(&db)
            .with_context(|| format!("인덱스를 열 수 없습니다: {}", db.display()))?;
        let graph = MemGraph::load(&store)?;
        let roots = pack::repo_roots(&config);
        Ok(Session {
            config_path: config_path.to_path_buf(),
            config,
            store,
            graph,
            roots,
        })
    }

    /// 설정에 저장된 랭킹 가중치. 팩 화면의 슬라이더가 이 값에서 시작한다.
    pub fn weights(&self) -> RankWeights {
        self.config.rank
    }

    /// 전문 검색. 도메인 용어 사전으로 질의를 넓힌 뒤 찾는다.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Hit>> {
        let expanded = self.config.semantic.expand_query(query);
        Ok(self
            .store
            .search(&expanded, limit)?
            .iter()
            .map(|h| hit(&h.node, h.score))
            .collect())
    }

    /// 고른 심볼과 이어져 있는 것들. 무엇이 함께 바뀌어야 하는지 보는 자리다.
    ///
    /// 호출·주입·API 호출·라우트 처리만 따라간다. 파일에 담겨 있다는 관계까지
    /// 넣으면 같은 파일의 심볼이 전부 딸려 나와 목록이 쓸모없어진다.
    pub fn neighbors(&self, id: &str, depth: u32) -> Result<Vec<Hit>> {
        let kinds = [
            EdgeKind::Calls,
            EdgeKind::Injects,
            EdgeKind::CallsApi,
            EdgeKind::Handles,
        ];
        Ok(self
            .store
            .neighbors(&NodeId(id.to_string()), &kinds, Direction::Both, depth)?
            .iter()
            .map(|n| hit(n, 0.0))
            .collect())
    }

    /// 태스크 문장 하나로 컨텍스트 팩을 만든다.
    pub fn pack(&self, task: &str, budget: usize, weights: RankWeights) -> Result<PackView> {
        let opts = pack::PackOptions {
            budget,
            weights,
            synonyms: self.config.semantic.clone(),
            ..Default::default()
        };
        let built = pack::build_pack(&self.store, &self.graph, task, &self.roots, &opts)?;
        let text = pack::render_text(&built);
        Ok(PackView { pack: built, text })
    }
}
