//! `nunchi.toml` — 솔루션별 설정 (docs/GUIDE.md 최초 적용)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "nunchi.toml";
/// 저장소에 커밋하는 공용 설정. 경로가 없으므로 머신이 달라도 그대로 쓴다.
pub const SHARED_FILE: &str = "nunchi.shared.toml";

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
    /// 도메인 용어 사전. 프레임워크 규칙과 같은 이유로 데이터에 둔다.
    #[serde(default)]
    pub semantic: crate::semantic::Synonyms,
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
    /// 생성 코드·벤더 디렉터리가 들어오면 랭킹이 오염된다 (docs/GUIDE.md 최초 적용).
    pub exclude: Vec<String>,
    pub max_file_bytes: u64,
    /// git 이력을 몇 커밋까지 읽을지. 0이면 이력 인덱싱을 건너뛴다.
    #[serde(default = "default_max_commits")]
    pub max_commits: usize,
}

fn default_max_commits() -> usize {
    1000
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig {
            languages: vec!["java".into(), "typescript".into(), "rust".into()],
            exclude: DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect(),
            max_file_bytes: 2 * 1024 * 1024,
            max_commits: default_max_commits(),
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

/// 랭킹 가중치 α~ε. 재컴파일 없이 조정하기 위해 설정으로 분리한다(docs/DESIGN.md 11절).
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

/// 저장소에 커밋하는 부분 — 경로가 들어가지 않는다.
///
/// 랭킹 가중치와 프레임워크 규칙은 **양쪽 머신이 같은 값을 써야 한다**
/// (docs/CONTRIBUTING.md 개발 환경). 반면 저장소 경로는 머신마다 다르다. 한 파일에 섞여 있으면
/// 통째로 gitignore할 수밖에 없어 가중치 공유가 불가능해진다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SharedConfig {
    #[serde(default)]
    pub index: Option<SharedIndex>,
    #[serde(default)]
    pub rank: Option<RankWeights>,
    #[serde(default)]
    pub framework: Option<crate::rules::FrameworkRules>,
    #[serde(default)]
    pub semantic: Option<crate::semantic::Synonyms>,
}

/// 공용 인덱싱 설정 — 경로를 담지 않는 항목만.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedIndex {
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub max_commits: Option<usize>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("설정 파일을 읽을 수 없습니다: {}", path.display()))?;
        let mut config: Config = toml::from_str(&text)
            .with_context(|| format!("설정 파일 파싱 실패: {}", path.display()))?;

        // 옆에 공용 설정이 있으면 덮어쓴다. 저장소에 커밋된 값이 기준이 된다.
        let shared_path = path.with_file_name(SHARED_FILE);
        if shared_path.is_file() {
            let shared: SharedConfig = toml::from_str(
                &std::fs::read_to_string(&shared_path)
                    .with_context(|| format!("공용 설정을 읽을 수 없습니다: {}", shared_path.display()))?,
            )
            .with_context(|| format!("공용 설정 파싱 실패: {}", shared_path.display()))?;
            config.apply_shared(shared);
        }
        Ok(config)
    }

    fn apply_shared(&mut self, shared: SharedConfig) {
        if let Some(i) = shared.index {
            if let Some(v) = i.languages {
                self.index.languages = v;
            }
            if let Some(v) = i.exclude {
                self.index.exclude = v;
            }
            if let Some(v) = i.max_commits {
                self.index.max_commits = v;
            }
        }
        if let Some(v) = shared.rank {
            self.rank = v;
        }
        if let Some(v) = shared.framework {
            self.framework = v;
        }
        if let Some(v) = shared.semantic {
            self.semantic = v;
        }
    }

    /// 공용으로 뽑아낼 부분만 추린다. TUI의 가중치 저장이 이걸 쓴다.
    pub fn to_shared(&self) -> SharedConfig {
        SharedConfig {
            index: Some(SharedIndex {
                languages: Some(self.index.languages.clone()),
                exclude: Some(self.index.exclude.clone()),
                max_commits: Some(self.index.max_commits),
            }),
            rank: Some(self.rank),
            framework: Some(self.framework.clone()),
            semantic: Some(self.semantic.clone()),
        }
    }

    pub fn save_shared(&self, dir: &Path) -> Result<PathBuf> {
        let path = dir.join(SHARED_FILE);
        std::fs::write(&path, toml::to_string_pretty(&self.to_shared())?)
            .with_context(|| format!("공용 설정을 쓸 수 없습니다: {}", path.display()))?;
        Ok(path)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nunchi-cfg-{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn shared_overrides_machine_local() -> Result<()> {
        let dir = tmpdir("shared");
        std::fs::write(
            dir.join(CONFIG_FILE),
            r#"
[solution]
name = "web"
repos = ["/machine/specific/path"]

[rank]
alpha_bm25 = 0.1
beta_ppr = 0.1
gamma_recency = 0.1
delta_cochange = 0.1
epsilon_central = 0.1
"#,
        )?;
        std::fs::write(
            dir.join(SHARED_FILE),
            r#"
[rank]
alpha_bm25 = 0.9
beta_ppr = 0.8
gamma_recency = 0.3
delta_cochange = 0.4
epsilon_central = 0.2

[semantic.terms]
"주문" = ["order"]
"#,
        )?;

        let c = Config::load(&dir.join(CONFIG_FILE))?;
        // 경로는 머신 로컬 값이 남고
        assert_eq!(c.solution.repos[0].to_string_lossy(), "/machine/specific/path");
        // 가중치·용어는 공용 값이 이긴다
        assert_eq!(c.rank.alpha_bm25, 0.9);
        assert!(c.semantic.terms.contains_key("주문"));
        Ok(())
    }

    #[test]
    fn shared_is_optional() -> Result<()> {
        let dir = tmpdir("nosh");
        std::fs::write(
            dir.join(CONFIG_FILE),
            "[solution]\nname=\"x\"\nrepos=[\"/a\"]\n",
        )?;
        let c = Config::load(&dir.join(CONFIG_FILE))?;
        assert_eq!(c.rank.alpha_bm25, RankWeights::default().alpha_bm25);
        Ok(())
    }

    #[test]
    fn shared_roundtrip_has_no_paths() -> Result<()> {
        let dir = tmpdir("roundtrip");
        std::fs::write(dir.join(CONFIG_FILE), "[solution]\nname=\"x\"\nrepos=[\"/secret/path\"]\n")?;
        let c = Config::load(&dir.join(CONFIG_FILE))?;
        let path = c.save_shared(&dir)?;
        let text = std::fs::read_to_string(&path)?;
        // 공용 파일에 머신 경로가 새어 나가면 안 된다.
        assert!(!text.contains("/secret/path"), "경로가 공용 설정에 들어갔다:\n{text}");
        Ok(())
    }
}
