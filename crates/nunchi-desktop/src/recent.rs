//! 최근에 연 솔루션 목록.
//!
//! CLI는 현재 디렉터리에서 위로 올라가며 설정 파일을 찾지만, 데스크톱 앱은
//! 어디서 실행될지 알 수 없다. Finder에서 아이콘을 더블클릭하면 현재
//! 디렉터리가 홈이나 루트가 되므로 그 방식으로는 설정을 찾지 못한다.
//! 그래서 열었던 솔루션을 기억해 두고 다음에 다시 연다.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 목록에 남길 최대 개수. 이보다 많아지면 오래된 것부터 버린다.
const MAX_ENTRIES: usize = 12;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    /// `nunchi.toml`의 절대 경로
    pub config_path: String,
    /// 솔루션 이름. 목록에 보여 준다.
    pub name: String,
    /// 마지막으로 연 시각(유닉스 초). 최신 순으로 정렬하는 데 쓴다.
    pub opened_at: u64,
    /// 설정 파일이 아직 그 자리에 있는가. 목록을 읽을 때마다 다시 확인한다.
    #[serde(default)]
    pub exists: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct File {
    #[serde(default)]
    solutions: Vec<Entry>,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 목록 파일의 위치. 앱 데이터 디렉터리 안에 둔다.
fn list_path(dir: &Path) -> PathBuf {
    dir.join("recent.json")
}

/// 최근 목록을 읽는다. 파일이 없거나 깨져 있으면 빈 목록으로 시작한다.
///
/// 읽으면서 설정 파일이 아직 있는지 확인해 `exists`를 채운다. 저장소를
/// 옮기거나 지웠을 수 있으므로 저장된 값을 믿지 않는다.
pub fn load(app_dir: &Path) -> Vec<Entry> {
    let Ok(text) = std::fs::read_to_string(list_path(app_dir)) else {
        return Vec::new();
    };
    let file: File = serde_json::from_str(&text).unwrap_or_default();
    let mut list = file.solutions;
    for e in list.iter_mut() {
        e.exists = Path::new(&e.config_path).is_file();
    }
    list.sort_by(|a, b| b.opened_at.cmp(&a.opened_at));
    list
}

/// 방금 연 솔루션을 목록 맨 앞으로 올린다.
pub fn touch(app_dir: &Path, config_path: &Path, name: &str) -> anyhow::Result<()> {
    let mut list = load(app_dir);
    let path = config_path.display().to_string();
    list.retain(|e| e.config_path != path);
    list.insert(
        0,
        Entry {
            config_path: path,
            name: name.to_string(),
            opened_at: now(),
            exists: true,
        },
    );
    list.truncate(MAX_ENTRIES);
    save(app_dir, &list)
}

/// 목록에서 하나를 지운다. 설정 파일 자체는 건드리지 않는다.
pub fn remove(app_dir: &Path, config_path: &str) -> anyhow::Result<Vec<Entry>> {
    let mut list = load(app_dir);
    list.retain(|e| e.config_path != config_path);
    save(app_dir, &list)?;
    Ok(list)
}

fn save(app_dir: &Path, list: &[Entry]) -> anyhow::Result<()> {
    std::fs::create_dir_all(app_dir)?;
    let file = File {
        solutions: list.to_vec(),
    };
    std::fs::write(list_path(app_dir), serde_json::to_string_pretty(&file)?)?;
    Ok(())
}
