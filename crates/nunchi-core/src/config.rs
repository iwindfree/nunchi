//! `nunchi.toml` — 솔루션별 설정 (PLAN.md 3.8절)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "nunchi.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub solution: Solution,
    #[serde(default)]
    pub index: IndexConfig,
    #[serde(default)]
    pub rank: RankWeights,
    /// 프레임워크 의미론 규칙. 비워두면 내장 기본값(Spring + React)이 쓰인다.
    /// 여기에 추가하면 재빌드 없이 지원 범위가 넓어진다 — `crate::rules` 참조.
    #[serde(default)]
    pub framework: crate::rules::FrameworkRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub name: String,
    /// 하나의 솔루션을 이루는 저장소들. 여러 개면 교차 저장소 엣지 대상이 된다.
    pub repos: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub languages: Vec<String>,
    /// 생성 코드·벤더 디렉터리가 들어오면 랭킹이 오염된다 (PLAN.md 3.8절).
    pub exclude: Vec<String>,
    pub max_file_bytes: u64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig {
            languages: vec!["java".into(), "typescript".into(), "rust".into()],
            exclude: DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect(),
            max_file_bytes: 2 * 1024 * 1024,
        }
    }
}

/// 디렉터리 가지치기가 되도록 `**/name` 형태(디렉터리 자체)와
/// `**/name/**` 형태(내부 파일)를 함께 넣는다.
pub const DEFAULT_EXCLUDES: &[&str] = &[
    "**/node_modules",
    "**/node_modules/**",
    "**/target",
    "**/target/**",
    "**/build",
    "**/build/**",
    "**/dist",
    "**/dist/**",
    "**/.next",
    "**/.next/**",
    "**/vendor",
    "**/vendor/**",
    "**/generated",
    "**/generated/**",
    "**/*.min.js",
    "**/*.lock",
];

/// 랭킹 가중치 α~ε. 재컴파일 없이 조정하기 위해 설정으로 분리한다(PLAN.md 1.6절).
/// TUI 팩 미리보기에서 실시간으로 만지고 저장한다.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RankWeights {
    pub alpha_bm25: f32,
    pub beta_ppr: f32,
    pub gamma_recency: f32,
    pub delta_cochange: f32,
    pub epsilon_central: f32,
}

impl Default for RankWeights {
    fn default() -> Self {
        RankWeights {
            alpha_bm25: 0.7,
            beta_ppr: 0.5,
            gamma_recency: 0.3,
            delta_cochange: 0.4,
            epsilon_central: 0.2,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("설정 파일을 읽을 수 없습니다: {}", path.display()))?;
        toml::from_str(&text)
            .with_context(|| format!("설정 파일 파싱 실패: {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)
            .with_context(|| format!("설정 파일을 쓸 수 없습니다: {}", path.display()))?;
        Ok(())
    }

    /// 현재 디렉터리부터 위로 올라가며 `nunchi.toml`을 찾는다.
    pub fn discover(start: &Path) -> Option<PathBuf> {
        let mut dir = Some(start);
        while let Some(d) = dir {
            let candidate = d.join(CONFIG_FILE);
            if candidate.is_file() {
                return Some(candidate);
            }
            dir = d.parent();
        }
        None
    }
}
