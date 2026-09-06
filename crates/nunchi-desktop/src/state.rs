//! 화면에 보여 줄 상태를 모은다.
//!
//! 로직은 전부 `nunchi-core`에 있고 여기서는 그것을 불러 화면이 쓰기 좋은
//! 형태로 옮기기만 한다. CLI와 MCP 서버가 같은 함수를 부르므로 세 통로가
//! 같은 결과를 낸다.

use nunchi_core::config::{CONFIG_FILE, Config};
use nunchi_core::store::sqlite::SqliteStore;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Default)]
pub struct Overview {
    pub config: Option<ConfigInfo>,
    pub index: Option<IndexInfo>,
    /// 설정이나 인덱스를 읽지 못한 이유. 화면에 그대로 보여 준다.
    pub problem: Option<String>,
}

#[derive(Serialize)]
pub struct ConfigInfo {
    pub path: String,
    pub solution: String,
    pub repos: Vec<RepoInfo>,
    pub languages: Vec<String>,
    pub max_candidates: usize,
    pub max_commits: usize,
    /// 프레임워크 규칙 수. 내장 기본값과 사용자 규칙을 합친 것이다.
    pub rule_count: usize,
}

#[derive(Serialize)]
pub struct RepoInfo {
    pub path: String,
    /// 경로가 실제로 있는가. 장비를 옮기면 어긋나므로 화면에서 알려 준다.
    pub exists: bool,
}

#[derive(Serialize)]
pub struct IndexInfo {
    pub path: String,
    pub nodes: i64,
    pub edges: i64,
    /// 인덱싱할 때 저장해 둔 지표. `nunchi doctor`가 읽는 것과 같다.
    pub metrics: serde_json::Value,
}

/// 설정 파일을 찾아 인덱스까지 읽는다.
/// 설정이 없으면 `config`가 비고, 인덱스가 없으면 `index`가 빈다.
pub fn overview() -> Overview {
    let Ok(cwd) = std::env::current_dir() else {
        return Overview {
            problem: Some("현재 디렉터리를 알 수 없습니다.".into()),
            ..Default::default()
        };
    };
    let Some(config_path) = Config::discover(&cwd) else {
        return Overview {
            problem: Some(format!(
                "{CONFIG_FILE}을 찾지 못했습니다. 저장소를 등록하면 시작할 수 있습니다."
            )),
            ..Default::default()
        };
    };
    let config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return Overview {
                problem: Some(format!("설정 파일을 읽지 못했습니다: {e}")),
                ..Default::default()
            };
        }
    };

    let rules = nunchi_core::rules::FrameworkRules::effective(&config.framework);
    let info = ConfigInfo {
        path: config_path.display().to_string(),
        solution: config.solution.name.clone(),
        repos: config
            .solution
            .repos
            .iter()
            .map(|r| RepoInfo {
                path: r.display().to_string(),
                exists: r.exists(),
            })
            .collect(),
        languages: config.index.languages.clone(),
        max_candidates: config.index.max_candidates,
        max_commits: config.index.max_commits,
        rule_count: rules.route.len()
            + rules.base_path.len()
            + rules.bean.len()
            + rules.inject.len()
            + rules.http_client.len()
            + rules.persistence.len(),
    };

    let db_path = index_path(&config_path);
    let index = read_index(&db_path);
    Overview {
        config: Some(info),
        index,
        problem: None,
    }
}

/// 인덱스는 설정 파일 옆의 `.nunchi/graph.db`에 있다.
fn index_path(config_path: &PathBuf) -> PathBuf {
    config_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join(".nunchi")
        .join("graph.db")
}

fn read_index(db_path: &PathBuf) -> Option<IndexInfo> {
    if !db_path.exists() {
        return None;
    }
    let store = SqliteStore::open(db_path).ok()?;
    Some(IndexInfo {
        path: db_path.display().to_string(),
        nodes: store.count_nodes().unwrap_or(0),
        edges: store.count_edges().unwrap_or(0),
        metrics: store
            .get_meta("metrics")
            .ok()
            .flatten()
            .and_then(|m| serde_json::from_str(&m).ok())
            .map(only_code_languages)
            .unwrap_or(serde_json::Value::Null),
    })
}

/// 언어 커버리지에서 코드가 아닌 것을 뺀다.
///
/// 인덱싱은 마크다운이나 TOML 파일도 세지만 심볼을 뽑지 않으므로 파싱률이
/// 언제나 0이다. 그것을 그대로 보여 주면 추출기에 문제가 있는 것처럼 읽힌다.
/// `nunchi doctor`도 같은 기준으로 거른다.
fn only_code_languages(mut metrics: serde_json::Value) -> serde_json::Value {
    if let Some(list) = metrics.get_mut("by_lang").and_then(|v| v.as_array_mut()) {
        list.retain(|e| {
            e["lang"]
                .as_str()
                .is_some_and(nunchi_core::lang::is_code)
        });
    }
    metrics
}
