//! 콘텐츠 주소 추출 캐시 (PLAN.md 3.7절)
//!
//! 브랜치 전환의 재파싱 비용을 없앤다. 핵심 통찰은 **A 계층(파일 내부 사실)이
//! 브랜치의 함수가 아니라 내용의 함수**라는 것이다:
//!
//! ```text
//! parse(blob_content) → { symbols, edges, spans }   ← 브랜치와 무관
//! ```
//!
//! 그래서 브랜치가 아니라 내용 해시로 캐싱한다. git이 blob을 다루는 방식 그대로다.
//! main → feature → main 왕복에서 복귀 시 파싱 0회가 된다.
//!
//! 캐시 키는 **git blob SHA가 아니라 워킹트리 파일 내용 해시**다.
//! `core.autocrlf=true` 인 Windows에서는 두 값이 갈리기 때문이다(PLAN.md 3.10절).

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS extract_cache (
    hash       TEXT PRIMARY KEY,
    lang       TEXT NOT NULL,
    payload    TEXT NOT NULL,
    bytes      INTEGER NOT NULL,
    used_at    INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS extract_cache_used ON extract_cache(used_at);
"#;

pub struct ExtractCache {
    conn: Connection,
    pub hits: usize,
    pub misses: usize,
}

impl ExtractCache {
    /// 저장소 인덱스와 **별도 파일**로 둔다. 워크트리마다 인덱스는 달라도
    /// 캐시는 공유해야 하기 때문이다(PLAN.md 3.7절).
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(SCHEMA)?;
        Ok(ExtractCache { conn, hits: 0, misses: 0 })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(ExtractCache { conn, hits: 0, misses: 0 })
    }

    pub fn get(&mut self, hash: &str, lang: &str) -> Option<String> {
        let found: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM extract_cache WHERE hash = ?1 AND lang = ?2",
                params![hash, lang],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        match found {
            Some(payload) => {
                self.hits += 1;
                let _ = self.conn.execute(
                    "UPDATE extract_cache SET used_at = strftime('%s','now') WHERE hash = ?1",
                    params![hash],
                );
                Some(payload)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    pub fn put(&mut self, hash: &str, lang: &str, payload: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO extract_cache (hash, lang, payload, bytes, used_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))
             ON CONFLICT(hash) DO UPDATE SET used_at = excluded.used_at",
            params![hash, lang, payload, payload.len() as i64],
        )?;
        Ok(())
    }

    /// 상한을 넘으면 오래 안 쓴 것부터 버린다.
    pub fn evict(&mut self, max_bytes: i64) -> Result<usize> {
        let total: i64 = self
            .conn
            .query_row("SELECT COALESCE(SUM(bytes),0) FROM extract_cache", [], |r| r.get(0))?;
        if total <= max_bytes {
            return Ok(0);
        }
        let removed = self.conn.execute(
            "DELETE FROM extract_cache WHERE hash IN (
                 SELECT hash FROM extract_cache ORDER BY used_at ASC LIMIT
                 (SELECT COUNT(*) / 4 FROM extract_cache)
             )",
            [],
        )?;
        Ok(removed)
    }

    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_hit_rate() -> Result<()> {
        let mut c = ExtractCache::open_in_memory()?;
        assert!(c.get("h1", "rust").is_none());
        c.put("h1", "rust", "{\"symbols\":[]}")?;
        assert_eq!(c.get("h1", "rust").as_deref(), Some("{\"symbols\":[]}"));
        // 언어가 다르면 다른 항목이다 — 같은 바이트라도 파서가 다르면 결과가 다르다.
        assert!(c.get("h1", "java").is_none());
        assert!(c.hit_rate() > 0.0 && c.hit_rate() < 1.0);
        Ok(())
    }

    #[test]
    fn eviction_frees_space() -> Result<()> {
        let mut c = ExtractCache::open_in_memory()?;
        for i in 0..40 {
            c.put(&format!("h{i}"), "rust", &"x".repeat(100))?;
        }
        assert!(c.evict(1000)? > 0);
        Ok(())
    }
}
