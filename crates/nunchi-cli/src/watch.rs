//! 파일 워처 — 증분 재인덱싱 (docs/DESIGN.md 8절)
//!
//! **checkout 자체는 거의 공짜여야 한다.** git checkout은 파일 수천 개를 한 번에
//! 바꾸므로 개별 이벤트로 처리하면 이벤트 폭풍 + 중간 상태 인덱싱으로 무너진다.
//! debounce로 묶고, 그 사이 또 변경이 오면 목록을 병합한다(docs/DESIGN.md 9절).

use anyhow::{Context, Result};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use nunchi_core::cache::ExtractCache;
use nunchi_core::{index, Config, SqliteStore};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 변경이 멎기를 기다리는 시간. 브랜치 전환의 이벤트 폭풍을 하나로 묶는다.
const DEBOUNCE: Duration = Duration::from_millis(500);

pub fn run(config: Config, db_path: PathBuf, cache_path: PathBuf) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Event>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .context("파일 워처를 만들 수 없습니다")?;

    for repo in &config.solution.repos {
        let root = repo
            .canonicalize()
            .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", repo.display()))?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .with_context(|| format!("감시 실패: {}", root.display()))?;
        println!("감시 중  {}", root.display());
    }
    println!("변경을 기다립니다. 중단하려면 Ctrl-C.\n");

    let mut pending: HashSet<PathBuf> = HashSet::new();
    let mut last_event: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                if !matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    continue;
                }
                for p in event.paths {
                    // 인덱스 자신의 변경에 반응하면 무한 루프가 된다.
                    if p.components().any(|c| c.as_os_str() == ".nunchi" || c.as_os_str() == ".git")
                    {
                        continue;
                    }
                    pending.insert(p);
                }
                if !pending.is_empty() {
                    last_event = Some(Instant::now());
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // debounce가 끝나야 실제 인덱싱을 한다.
        let ready = last_event.is_some_and(|t| t.elapsed() >= DEBOUNCE);
        if !ready || pending.is_empty() {
            continue;
        }

        let count = pending.len();
        pending.clear();
        last_event = None;

        let started = Instant::now();
        // 현재는 전체 재인덱싱이되, 콘텐츠 주소 캐시가 재파싱을 막아 비용이 낮다.
        // 파일 단위 부분 갱신은 다음 단계다.
        match reindex(&config, &db_path, &cache_path) {
            Ok(stats) => println!(
                "변경 {count}건 → 재인덱싱 {:.2}s · 캐시 적중 {}/{} ({:.0}%)",
                started.elapsed().as_secs_f64(),
                stats.cache_hits,
                stats.cache_hits + stats.cache_misses,
                if stats.cache_hits + stats.cache_misses > 0 {
                    stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0
                } else {
                    0.0
                }
            ),
            Err(e) => eprintln!("재인덱싱 실패: {e}"),
        }
    }
    Ok(())
}

fn reindex(config: &Config, db_path: &PathBuf, cache_path: &PathBuf) -> Result<index::IndexStats> {
    let mut store = SqliteStore::open(db_path)?;
    let mut cache = ExtractCache::open(cache_path)?;
    store.clear()?;
    let stats = index::index_all_with_cache(config, &mut store, Some(&mut cache))?;
    // 캐시가 무한정 자라지 않게 한다.
    cache.evict(2 * 1024 * 1024 * 1024)?;
    Ok(stats)
}
