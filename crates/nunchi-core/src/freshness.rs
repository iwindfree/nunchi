//! 인덱스가 실제 코드와 얼마나 어긋났는지 잰다.
//!
//! **인덱스는 낡는다.** 워처가 꺼져 있는 동안 터미널에서 `git pull`을 하거나
//! 다른 편집기로 고치면 우리는 그 사실을 모른다.
//!
//! 팩은 낡은 항목을 버리므로 틀린 좌표를 주지는 않는다. 그런데 **말없이 적게
//! 준다.** 새로 생긴 파일은 아예 나오지 않는다. 에이전트는 자기가 무엇을 받지
//! 못했는지 알 방법이 없고, 사람도 결과가 부실하다고만 느낀다.
//!
//! 그래서 재서 알린다. 고치지는 않는다. 다시 인덱싱할지는 사람이 정한다.

use crate::config::Config;
use crate::store::sqlite::{IndexedFile, SqliteStore};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

/// 인덱스와 실제 코드의 차이.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Drift {
    /// 인덱싱한 뒤 내용이 바뀐 파일
    pub changed: usize,
    /// 인덱스에 없는 파일
    pub added: usize,
    /// 인덱스에는 있는데 사라진 파일
    pub removed: usize,
    /// 인덱싱되어 있는 파일 수. 어긋난 비율을 가늠하는 기준이다.
    pub indexed: usize,
    /// 무엇이 어긋났는지 보여 줄 예시. 많아야 다섯 개다.
    pub examples: Vec<String>,
    /// 재는 데 걸린 밀리초. 이 검사가 비싸지면 여기서 드러난다.
    pub took_ms: u64,
}

/// 예시를 몇 개까지 모을지. 목록이 아니라 신호이므로 조금이면 된다.
const EXAMPLE_LIMIT: usize = 5;

impl Drift {
    pub fn is_behind(&self) -> bool {
        self.changed + self.added + self.removed > 0
    }

    /// 사람과 에이전트가 함께 읽는 한 줄. 어긋나지 않았으면 아무것도 없다.
    ///
    /// 무엇을 해야 하는지까지 적는다. "낡았다"만 알려 주면 받는 쪽이 다음
    /// 행동을 정할 수 없다.
    pub fn summary(&self) -> Option<String> {
        if !self.is_behind() {
            return None;
        }
        let mut parts = Vec::new();
        if self.changed > 0 {
            parts.push(format!("바뀐 파일 {}개", self.changed));
        }
        if self.added > 0 {
            parts.push(format!("새 파일 {}개", self.added));
        }
        if self.removed > 0 {
            parts.push(format!("사라진 파일 {}개", self.removed));
        }
        Some(format!(
            "인덱스가 실제 코드와 어긋나 있습니다({}). \
             결과에 빠진 것이 있을 수 있으니 `nunchi index`로 갱신하십시오.",
            parts.join(", ")
        ))
    }
}

/// 인덱스와 실제 코드를 맞대어 본다.
///
/// 수정 시각이 같으면 내용도 같다고 보고 넘어간다. 파일을 전부 읽어 해시하면
/// 이 검사가 인덱싱만큼 비싸져서 호출할 때마다 돌릴 수 없게 된다. 수정 시각이
/// 다를 때만 실제로 읽어 해시를 견준다. `git checkout`은 내용이 같아도 수정
/// 시각을 바꾸므로, 그 경우를 걸러 내려면 이 확인이 필요하다.
pub fn measure(config: &Config, store: &SqliteStore) -> Result<Drift> {
    let started = std::time::Instant::now();
    let excludes = crate::index::build_exclude_set(&config.index.exclude)?;

    // 인덱스에 있는 파일을 (저장소, 경로)로 찾을 수 있게 편다.
    let mut indexed: HashMap<(String, String), IndexedFile> = store
        .indexed_files()?
        .into_iter()
        .map(|f| ((f.repo.clone(), f.path.clone()), f))
        .collect();

    let mut drift = Drift {
        indexed: indexed.len(),
        ..Default::default()
    };

    for root in &config.solution.repos {
        let repo = crate::index::repo_name(root);
        if !root.is_dir() {
            // 저장소 폴더가 통째로 사라졌다. 그 안의 것을 하나씩 세면
            // 사라진 파일 수천 개가 되어 무엇이 문제인지 오히려 흐려진다.
            drift
                .examples
                .push(format!("{} 폴더를 찾을 수 없습니다", root.display()));
            continue;
        }

        for entry in crate::index::source_walker(root, &excludes) {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let abs = entry.path();
            let Some(rel) = crate::path::relative_to(root, abs) else {
                continue;
            };
            if excludes.is_match(&rel) {
                continue;
            }
            if crate::lang::detect(abs).is_none() {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            if meta.len() > config.index.max_file_bytes {
                continue;
            }

            match indexed.remove(&(repo.clone(), rel.clone())) {
                None => {
                    drift.added += 1;
                    note(&mut drift, format!("새 파일  {repo}/{rel}"));
                }
                Some(was) => {
                    if unchanged(&meta, &was) {
                        continue;
                    }
                    // 수정 시각이나 크기가 달라졌다고 내용까지 바뀐 것은 아니다.
                    // `git checkout`은 같은 내용을 다시 써서 시각만 바꾼다.
                    let Ok(bytes) = std::fs::read(crate::path::to_extended_length(abs)) else {
                        continue;
                    };
                    if was.hash.as_deref() != Some(crate::path::content_hash(&bytes).as_str()) {
                        drift.changed += 1;
                        note(&mut drift, format!("바뀜    {repo}/{rel}"));
                    }
                }
            }
        }
    }

    // 훑으면서 지우고 남은 것은 디스크에 없는 파일이다.
    //
    // 다만 설정에서 빠진 저장소의 파일도 여기 남는다. 그것은 파일이 사라진
    // 것이 아니라 더 이상 보지 않기로 한 것이므로 세지 않는다.
    let watched: Vec<String> = config
        .solution
        .repos
        .iter()
        .map(|r| crate::index::repo_name(r))
        .collect();
    for (repo, rel) in indexed.into_keys() {
        if !watched.contains(&repo) {
            continue;
        }
        drift.removed += 1;
        note(&mut drift, format!("사라짐  {repo}/{rel}"));
    }

    drift.took_ms = started.elapsed().as_millis() as u64;
    Ok(drift)
}

fn note(drift: &mut Drift, line: String) {
    if drift.examples.len() < EXAMPLE_LIMIT {
        drift.examples.push(line);
    }
}

/// 읽어 보지 않고도 그대로라고 볼 수 있는가.
///
/// 수정 시각과 크기가 모두 같아야 한다. 수정 시각은 초 단위라 같은 초 안에
/// 두 번 고치면 구별되지 않는데, 그때 길이가 달라졌다면 크기에서 드러난다.
///
/// 인덱서와 **같은 방식으로** 초 단위 값을 얻어야 한다. 한쪽이 밀리초를 쓰면
/// 모든 파일이 바뀐 것으로 나온다.
fn unchanged(meta: &std::fs::Metadata, was: &IndexedFile) -> bool {
    let Some(indexed_mtime) = was.mtime else {
        return false;
    };
    let same_mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .is_some_and(|now| now == indexed_mtime);
    // 크기는 예전 인덱스에 없다. 없으면 시각만 본다.
    let same_size = was.bytes.is_none_or(|bytes| bytes == meta.len());
    same_mtime && same_size
}

/// 마지막으로 잰 결과를 들고 있다가 너무 자주 다시 재지 않게 한다.
///
/// MCP 서버는 도구를 부를 때마다 이것을 본다. 매번 저장소를 훑으면 큰
/// 저장소에서 질의 자체보다 검사가 비싸진다.
pub struct Freshness {
    interval: std::time::Duration,
    last: std::sync::Mutex<Option<(std::time::Instant, Drift)>>,
}

impl Freshness {
    pub fn new(interval: std::time::Duration) -> Self {
        Freshness {
            interval,
            last: std::sync::Mutex::new(None),
        }
    }

    /// 필요하면 다시 재고, 아니면 들고 있던 값을 준다.
    ///
    /// 재는 데 실패하면 조용히 아무것도 돌려주지 않는다. 신선도를 알 수 없다는
    /// 이유로 질의를 막을 일은 아니다.
    pub fn get(&self, config: &Config, store: &SqliteStore) -> Option<Drift> {
        let mut slot = self.last.lock().ok()?;
        let fresh_enough = slot
            .as_ref()
            .is_some_and(|(at, _)| at.elapsed() < self.interval);
        if !fresh_enough {
            *slot = measure(config, store)
                .ok()
                .map(|d| (std::time::Instant::now(), d));
        }
        slot.as_ref().map(|(_, d)| d.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nunchi-fresh-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn config_for(dir: &Path) -> Config {
        let mut config = Config {
            solution: crate::config::Solution {
                name: "t".into(),
                repos: vec![dir.to_path_buf()],
            },
            index: Default::default(),
            rank: Default::default(),
            framework: Default::default(),
            semantic: Default::default(),
        };
        config.index.max_commits = 0;
        config
    }

    fn indexed(dir: &Path) -> (Config, SqliteStore) {
        let config = config_for(dir);
        let mut store = SqliteStore::open_in_memory().unwrap();
        crate::index::index_all(&config, &mut store).unwrap();
        (config, store)
    }

    /// 인덱싱 직후에는 어긋난 것이 없어야 한다.
    ///
    /// 이 테스트가 진짜 지키는 것은 숫자가 아니라 **거르는 규칙이 갈라지지
    /// 않는 것**이다. 신선도 검사가 인덱서와 다르게 거르기 시작하면 멀쩡한
    /// 파일이 새 파일로 잡혀 여기서 바로 터진다.
    #[test]
    fn a_fresh_index_reports_nothing() {
        let dir = temp_dir("clean");
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        std::fs::write(dir.join("b.py"), "def b():\n    pass\n").unwrap();
        // 인덱싱 대상이 아닌 것들. 이것들이 새 파일로 잡히면 안 된다.
        std::fs::write(dir.join("notes.txt"), "메모\n").unwrap();
        std::fs::create_dir_all(dir.join("node_modules")).unwrap();
        std::fs::write(dir.join("node_modules/x.js"), "x\n").unwrap();

        let (config, store) = indexed(&dir);
        let drift = measure(&config, &store).unwrap();
        assert!(
            !drift.is_behind(),
            "갓 인덱싱했는데 어긋났다고 한다: {drift:?}"
        );
        assert!(drift.indexed >= 2, "{drift:?}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn notices_changed_added_and_removed_files() {
        let dir = temp_dir("drift");
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        std::fs::write(dir.join("gone.rs"), "fn gone() {}\n").unwrap();
        let (config, store) = indexed(&dir);

        std::fs::write(dir.join("a.rs"), "fn a() { changed(); }\n").unwrap();
        std::fs::write(dir.join("new.rs"), "fn brand_new() {}\n").unwrap();
        std::fs::remove_file(dir.join("gone.rs")).unwrap();

        let drift = measure(&config, &store).unwrap();
        assert_eq!(
            (drift.changed, drift.added, drift.removed),
            (1, 1, 1),
            "{drift:?}"
        );
        assert!(drift.summary().is_some_and(|s| s.contains("nunchi index")));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 내용이 같은데 수정 시각만 바뀌는 일이 흔하다. `git checkout`이 그렇다.
    #[test]
    fn a_touched_file_with_the_same_content_is_not_drift() {
        let dir = temp_dir("touch");
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        let (config, store) = indexed(&dir);

        // 같은 내용을 다시 써서 수정 시각만 바꾼다.
        std::fs::write(dir.join("a.rs"), "fn a() {}\n").unwrap();
        let after = std::time::SystemTime::now() + std::time::Duration::from_secs(120);
        let _ = std::fs::File::open(dir.join("a.rs"))
            .and_then(|f| f.set_times(std::fs::FileTimes::new().set_modified(after)));

        let drift = measure(&config, &store).unwrap();
        assert!(
            !drift.is_behind(),
            "내용이 같은데 어긋났다고 한다: {drift:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
