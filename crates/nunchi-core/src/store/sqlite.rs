//! SQLite 저장소 구현 (PLAN.md 2절 결정)
//!
//! WAL 모드를 쓰는 이유는 인덱서(쓰기)와 MCP 서버(읽기)가 별도 프로세스이기 때문이다
//! (PLAN.md 3.5절). 임베디드 그래프 DB 다수가 갖는 단일 라이터 제약을 여기서 피한다.

use super::{RankOpts, Ranked, SearchHit, Store};
use crate::model::*;
use crate::path::compare_key;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::{HashSet, VecDeque};
use std::path::Path;

/// 스키마 버전. 올리면 인덱스를 자동 전체 재빌드한다(PLAN.md 3.6절).
pub const SCHEMA_VERSION: i64 = 1;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS nodes (
    id            TEXT PRIMARY KEY,
    key           TEXT NOT NULL,
    kind          TEXT NOT NULL,
    name          TEXT NOT NULL,
    repo          TEXT NOT NULL,
    path          TEXT,
    start_line    INTEGER,
    end_line      INTEGER,
    signature     TEXT,
    doc           TEXT,
    lang          TEXT,
    content_hash  TEXT,
    attrs         TEXT NOT NULL DEFAULT 'null'
);
CREATE INDEX IF NOT EXISTS nodes_key  ON nodes(key);
CREATE INDEX IF NOT EXISTS nodes_kind ON nodes(kind);
CREATE INDEX IF NOT EXISTS nodes_repo ON nodes(repo, path);
CREATE INDEX IF NOT EXISTS nodes_lang ON nodes(lang);

CREATE TABLE IF NOT EXISTS edges (
    src        TEXT NOT NULL,
    dst        TEXT NOT NULL,
    kind       TEXT NOT NULL,
    provenance TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    weight     REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (src, dst, kind)
);
CREATE INDEX IF NOT EXISTS edges_src ON edges(src, kind);
CREATE INDEX IF NOT EXISTS edges_dst ON edges(dst, kind);

-- 저장소별 HEAD. 분리 저장소에서 브랜치 편차를 감지하는 데 쓴다(PLAN.md 3.9절).
CREATE TABLE IF NOT EXISTS repos (
    repo   TEXT PRIMARY KEY,
    root   TEXT NOT NULL,
    branch TEXT,
    head   TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
    id UNINDEXED,
    name,
    signature,
    doc,
    path,
    tokenize = 'unicode61'
);
"#;

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)
            .with_context(|| format!("인덱스를 열 수 없습니다: {}", path.display()))?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        // WAL: 쓰기 중에도 읽기가 막히지 않는다 (PLAN.md 3.6절).
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(SCHEMA)?;

        let mut store = SqliteStore { conn };
        match store.schema_version()? {
            None => store.set_meta("schema_version", &SCHEMA_VERSION.to_string())?,
            Some(v) if v != SCHEMA_VERSION => {
                anyhow::bail!(
                    "인덱스 스키마 버전 불일치 (인덱스={v}, 기대={SCHEMA_VERSION}). \
                     `nunchi index --rebuild`로 재구축하세요."
                );
            }
            Some(_) => {}
        }
        Ok(store)
    }

    pub fn schema_version(&self) -> Result<Option<i64>> {
        Ok(self.get_meta("schema_version")?.and_then(|v| v.parse().ok()))
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
                r.get::<_, String>(0)
            })
            .optional()?)
    }

    pub fn set_meta(&mut self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn record_repo(&mut self, repo: &str, root: &str, branch: Option<&str>, head: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO repos (repo, root, branch, head) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(repo) DO UPDATE SET root=excluded.root, branch=excluded.branch, head=excluded.head",
            params![repo, root, branch, head],
        )?;
        Ok(())
    }

    pub fn get_node(&self, id: &NodeId) -> Result<Option<Node>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, kind, name, repo, path, start_line, end_line, signature, doc, lang, content_hash, attrs
                 FROM nodes WHERE id = ?1",
                params![id.as_str()],
                row_to_node,
            )
            .optional()?)
    }

    /// 파일 노드의 저장된 내용 해시. 지연 검증(PLAN.md 3.6절)에서 쓴다.
    pub fn file_hash(&self, repo: &str, path: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT content_hash FROM nodes WHERE kind = 'file' AND repo = ?1 AND key = ?2",
                params![repo, compare_key(path)],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten())
    }

    /// 종류만 싸게 조회 — 랭킹 루프에서 전체 노드를 로드하지 않기 위해서다.
    pub fn node_kind(&self, id: &NodeId) -> Result<Option<NodeKind>> {
        Ok(self
            .conn
            .query_row("SELECT kind FROM nodes WHERE id = ?1", params![id.as_str()], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
            .and_then(|k| NodeKind::parse(&k)))
    }

    pub fn count_nodes(&self) -> Result<i64> {
        Ok(self.conn.query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))?)
    }

    pub fn count_edges(&self) -> Result<i64> {
        Ok(self.conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))?)
    }

    /// 언어별 파일 수 — `nunchi doctor` 커버리지 표의 원천 (PLAN.md 3.8절).
    pub fn files_by_lang(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(lang, '(unknown)'), COUNT(*) FROM nodes
             WHERE kind = 'file' GROUP BY 1 ORDER BY 2 DESC",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// 메모리 그래프 적재용 — 전체 노드 ID (PLAN.md 2절)
    pub fn all_node_ids(&self) -> Result<Vec<NodeId>> {
        let mut stmt = self.conn.prepare("SELECT id FROM nodes")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0).map(NodeId))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// (src, dst, kind, confidence×weight)
    pub fn all_edges(&self) -> Result<Vec<(String, String, String, f32)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT src, dst, kind, confidence * weight FROM edges")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// 파일 노드의 최근 변경 시각(Unix). 랭킹의 recency 항에 쓴다.
    pub fn set_recency(&mut self, id: &NodeId, epoch: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE nodes SET attrs = json_set(COALESCE(NULLIF(attrs,'null'),'{}'), '$.mtime', ?2) WHERE id = ?1",
            params![id.as_str(), epoch],
        )?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM nodes; DELETE FROM edges; DELETE FROM nodes_fts; DELETE FROM repos;",
        )?;
        Ok(())
    }

    fn expand(
        &self,
        start: &NodeId,
        kinds: &[EdgeKind],
        dir: Direction,
        depth: u32,
    ) -> Result<Vec<NodeId>> {
        let kind_filter: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
        let mut seen: HashSet<String> = HashSet::from([start.0.clone()]);
        let mut frontier = vec![start.0.clone()];
        let mut out = Vec::new();

        for _ in 0..depth {
            let mut next = Vec::new();
            for node in &frontier {
                for neighbor in self.adjacent(node, &kind_filter, dir)? {
                    if seen.insert(neighbor.clone()) {
                        out.push(NodeId(neighbor.clone()));
                        next.push(neighbor);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        Ok(out)
    }

    fn adjacent(&self, node: &str, kinds: &[&str], dir: Direction) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let filter = if kinds.is_empty() {
            String::new()
        } else {
            let list: Vec<String> = kinds.iter().map(|k| format!("'{k}'")).collect();
            format!(" AND kind IN ({})", list.join(","))
        };

        if matches!(dir, Direction::Out | Direction::Both) {
            let sql = format!("SELECT dst FROM edges WHERE src = ?1{filter}");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![node], |r| r.get::<_, String>(0))?;
            out.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
        if matches!(dir, Direction::In | Direction::Both) {
            let sql = format!("SELECT src FROM edges WHERE dst = ?1{filter}");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![node], |r| r.get::<_, String>(0))?;
            out.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
        Ok(out)
    }

    fn load_nodes(&self, ids: &[NodeId]) -> Result<Vec<Node>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(node) = self.get_node(id)? {
                out.push(node);
            }
        }
        Ok(out)
    }
}

fn row_to_node(row: &Row<'_>) -> rusqlite::Result<Node> {
    let start: Option<u32> = row.get(5)?;
    let end: Option<u32> = row.get(6)?;
    let attrs: String = row.get(11)?;
    Ok(Node {
        id: NodeId(row.get(0)?),
        kind: NodeKind::parse(&row.get::<_, String>(1)?).unwrap_or(NodeKind::Symbol),
        name: row.get(2)?,
        repo: row.get(3)?,
        path: row.get(4)?,
        span: match (start, end) {
            (Some(s), Some(e)) => Some(Span { start_line: s, end_line: e }),
            _ => None,
        },
        signature: row.get(7)?,
        doc: row.get(8)?,
        lang: row.get(9)?,
        content_hash: row.get(10)?,
        attrs: serde_json::from_str(&attrs).unwrap_or(serde_json::Value::Null),
    })
}

/// FTS5 특수문자를 무력화한다. 사용자 질의를 그대로 넘기면 구문 오류가 난다.
fn fts_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ")
}

impl Store for SqliteStore {
    fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT INTO nodes (id, key, kind, name, repo, path, start_line, end_line,
                                    signature, doc, lang, content_hash, attrs)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                 ON CONFLICT(id) DO UPDATE SET
                    kind=excluded.kind, name=excluded.name, repo=excluded.repo,
                    path=excluded.path, start_line=excluded.start_line, end_line=excluded.end_line,
                    signature=excluded.signature, doc=excluded.doc, lang=excluded.lang,
                    content_hash=excluded.content_hash, attrs=excluded.attrs",
            )?;
            let mut del_fts = tx.prepare("DELETE FROM nodes_fts WHERE id = ?1")?;
            let mut ins_fts = tx.prepare(
                "INSERT INTO nodes_fts (id, name, signature, doc, path) VALUES (?1,?2,?3,?4,?5)",
            )?;

            for n in nodes {
                let key = compare_key(n.path.as_deref().unwrap_or(&n.name));
                ins.execute(params![
                    n.id.as_str(),
                    key,
                    n.kind.as_str(),
                    n.name,
                    n.repo,
                    n.path,
                    n.span.map(|s| s.start_line),
                    n.span.map(|s| s.end_line),
                    n.signature,
                    n.doc,
                    n.lang,
                    n.content_hash,
                    serde_json::to_string(&n.attrs)?,
                ])?;
                del_fts.execute(params![n.id.as_str()])?;
                ins_fts.execute(params![
                    n.id.as_str(),
                    n.name,
                    n.signature,
                    n.doc,
                    n.path
                ])?;
            }
        }
        tx.commit()?;
        Ok(nodes.len())
    }

    fn upsert_edges(&mut self, edges: &[Edge]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO edges (src, dst, kind, provenance, confidence, weight)
                 VALUES (?1,?2,?3,?4,?5,?6)
                 ON CONFLICT(src, dst, kind) DO UPDATE SET
                    provenance=excluded.provenance,
                    confidence=excluded.confidence,
                    weight=excluded.weight",
            )?;
            for e in edges {
                stmt.execute(params![
                    e.src.as_str(),
                    e.dst.as_str(),
                    e.kind.as_str(),
                    e.provenance.as_str(),
                    e.confidence,
                    e.weight
                ])?;
            }
        }
        tx.commit()?;
        Ok(edges.len())
    }

    fn neighbors(
        &self,
        id: &NodeId,
        kinds: &[EdgeKind],
        dir: Direction,
        depth: u32,
    ) -> Result<Vec<Node>> {
        let ids = self.expand(id, kinds, dir, depth.max(1))?;
        self.load_nodes(&ids)
    }

    fn paths(&self, from: &NodeId, to: &NodeId, max_depth: u32) -> Result<Vec<Vec<NodeId>>> {
        // BFS 최단 경로 1개. 다중 경로는 Phase 3(cg_impact 고도화)에서 확장한다.
        let mut queue = VecDeque::from([vec![from.clone()]]);
        let mut seen: HashSet<String> = HashSet::from([from.0.clone()]);

        while let Some(path) = queue.pop_front() {
            if path.len() as u32 > max_depth + 1 {
                break;
            }
            let tail = path.last().expect("경로는 비어 있지 않다");
            if tail == to {
                return Ok(vec![path]);
            }
            for next in self.adjacent(tail.as_str(), &[], Direction::Out)? {
                if seen.insert(next.clone()) {
                    let mut extended = path.clone();
                    extended.push(NodeId(next));
                    queue.push_back(extended);
                }
            }
        }
        Ok(Vec::new())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.kind, n.name, n.repo, n.path, n.start_line, n.end_line,
                    n.signature, n.doc, n.lang, n.content_hash, n.attrs,
                    -- 컬럼 가중치: name > signature > doc > path.
                    -- 경로에는 디렉터리 이름이 잔뜩 들어 있어 가중치를 주면
                    -- 파일 노드가 상위를 점령한다(실측에서 확인).
                    bm25(nodes_fts, 0.0, 10.0, 3.0, 2.0, 0.5) AS score
             FROM nodes_fts
             JOIN nodes n ON n.id = nodes_fts.id
             WHERE nodes_fts MATCH ?1
             ORDER BY score
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![fts_query(query), limit as i64], |row| {
            let score: f64 = row.get(12)?;
            Ok(SearchHit {
                node: row_to_node(row)?,
                // bm25()는 관련성이 높을수록 더 음수다. 부호를 뒤집어 직관적으로 만든다.
                score: -score as f32,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn rank(&self, _seeds: &[NodeId], _opts: &RankOpts) -> Result<Vec<Ranked>> {
        anyhow::bail!(
            "rank()는 Phase 2에서 구현합니다 (PLAN.md 4절). \
             메모리 인접리스트 기반 Personalized PageRank가 필요합니다."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file(repo: &str, path: &str, lang: &str) -> Node {
        let mut n = Node::new(NodeId::file(repo, path), NodeKind::File, path, repo);
        n.path = Some(path.to_string());
        n.lang = Some(lang.to_string());
        n
    }

    #[test]
    fn upsert_is_idempotent() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        let node = sample_file("api", "src/OrderService.java", "java");
        store.upsert_nodes(std::slice::from_ref(&node))?;
        store.upsert_nodes(std::slice::from_ref(&node))?;
        assert_eq!(store.count_nodes()?, 1);
        Ok(())
    }

    #[test]
    fn search_finds_by_name() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        store.upsert_nodes(&[
            sample_file("api", "src/OrderService.java", "java"),
            sample_file("web", "src/hooks/useOrder.ts", "typescript"),
        ])?;
        let hits = store.search("OrderService", 10)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.repo, "api");
        Ok(())
    }

    #[test]
    fn search_tolerates_fts_metacharacters() -> Result<()> {
        let store = SqliteStore::open_in_memory()?;
        // 따옴표·연산자가 섞여도 구문 오류로 죽지 않아야 한다.
        assert!(store.search("order AND \"(", 5).is_ok());
        Ok(())
    }

    #[test]
    fn neighbors_walks_edges() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        let a = sample_file("api", "a.java", "java");
        let b = sample_file("api", "b.java", "java");
        let c = sample_file("api", "c.java", "java");
        store.upsert_nodes(&[a.clone(), b.clone(), c.clone()])?;
        store.upsert_edges(&[
            Edge::new(a.id.clone(), b.id.clone(), EdgeKind::Calls, Provenance::Fast),
            Edge::new(b.id.clone(), c.id.clone(), EdgeKind::Calls, Provenance::Fast),
        ])?;

        let one = store.neighbors(&a.id, &[EdgeKind::Calls], Direction::Out, 1)?;
        assert_eq!(one.len(), 1);
        let two = store.neighbors(&a.id, &[EdgeKind::Calls], Direction::Out, 2)?;
        assert_eq!(two.len(), 2);
        Ok(())
    }

    #[test]
    fn paths_finds_a_route() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        let a = sample_file("api", "a.java", "java");
        let b = sample_file("api", "b.java", "java");
        store.upsert_nodes(&[a.clone(), b.clone()])?;
        store.upsert_edges(&[Edge::new(
            a.id.clone(),
            b.id.clone(),
            EdgeKind::Calls,
            Provenance::Fast,
        )])?;
        let found = store.paths(&a.id, &b.id, 3)?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].len(), 2);
        Ok(())
    }
}
