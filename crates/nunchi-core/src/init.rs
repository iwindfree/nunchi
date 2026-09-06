//! 솔루션 초기화 — 저장소를 훑어 언어를 감지하고 설정 파일을 만든다.
//!
//! CLI의 `nunchi init`과 데스크톱 앱의 초기 설정 화면이 이 함수를 함께 쓴다.
//! 두 곳에서 따로 구현하면 감지 결과나 기본값이 어긋난다.

use crate::config::{CONFIG_FILE, Config, DEFAULT_EXCLUDES, IndexConfig, RankWeights, Solution};
use crate::{index, lang, path as npath};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 초기화 결과. 화면이나 터미널에 무엇을 만들었는지 알려 주는 데 쓴다.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InitOutcome {
    pub config_path: PathBuf,
    pub shared_path: PathBuf,
    pub solution: String,
    pub repos: Vec<PathBuf>,
    pub languages: Vec<String>,
    /// 언어를 하나도 감지하지 못해 기본값을 넣었는가.
    pub used_default_languages: bool,
}

/// 설정 파일 두 개를 만든다.
///
/// `dir`은 설정을 둘 디렉터리이고, `repos`는 인덱싱 대상 저장소들이다.
/// `name`을 생략하면 첫 저장소의 디렉터리 이름을 쓴다.
pub fn init_solution(
    dir: &Path,
    repos: &[PathBuf],
    name: Option<String>,
    force: bool,
) -> Result<InitOutcome> {
    let target = dir.join(CONFIG_FILE);
    if target.exists() && !force {
        anyhow::bail!("{CONFIG_FILE}이 이미 있습니다. 덮어쓰려면 force를 지정하세요.");
    }

    let mut resolved = Vec::new();
    for r in repos {
        resolved.push(
            r.canonicalize()
                .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", r.display()))?,
        );
    }
    if resolved.is_empty() {
        anyhow::bail!("저장소를 하나 이상 지정해야 합니다.");
    }

    let detected = detect_languages(&resolved)?;
    let used_default = detected.is_empty();
    let solution_name = name.unwrap_or_else(|| {
        resolved[0]
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "solution".into())
    });

    let config = Config {
        solution: Solution {
            name: solution_name.clone(),
            repos: resolved.clone(),
        },
        index: IndexConfig {
            languages: if used_default {
                IndexConfig::default().languages
            } else {
                detected.clone()
            },
            ..IndexConfig::default()
        },
        rank: RankWeights::default(),
        // 비워두면 내장 규칙(Spring + React)이 적용된다.
        framework: Default::default(),
        semantic: Default::default(),
    };
    config.save(&target)?;
    // 공용 설정도 함께 만든다. 경로가 없으므로 커밋해서 공유한다.
    let shared_path = config.save_shared(dir)?;

    Ok(InitOutcome {
        config_path: target,
        shared_path,
        solution: solution_name,
        repos: resolved,
        languages: if used_default {
            config.index.languages.clone()
        } else {
            detected
        },
        used_default_languages: used_default,
    })
}

/// 저장소를 훑어 실제로 존재하는 코드 언어를 찾는다.
pub fn detect_languages(repos: &[PathBuf]) -> Result<Vec<String>> {
    let excludes = index::build_exclude_set(
        &DEFAULT_EXCLUDES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )?;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();

    for root in repos {
        for entry in ignore::WalkBuilder::new(root).hidden(true).build().flatten() {
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let Some(rel) = npath::relative_to(root, entry.path()) else {
                continue;
            };
            if excludes.is_match(&rel) {
                continue;
            }
            if let Some(l) = lang::detect(entry.path()) {
                if lang::is_code(l) {
                    *counts.entry(l).or_default() += 1;
                }
            }
        }
    }

    let mut langs: Vec<_> = counts.into_iter().collect();
    langs.sort_by(|a, b| b.1.cmp(&a.1));
    // 파일이 극소수인 언어는 노이즈일 가능성이 크다.
    Ok(langs
        .into_iter()
        .filter(|(_, n)| *n >= 3)
        .map(|(l, _)| l.to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트마다 다른 이름을 써야 서로 간섭하지 않는다.
    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nunchi-init-{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn repo_with(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = scratch(name);
        for (file, body) in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }
        dir
    }

    #[test]
    fn detects_the_language_with_enough_files() {
        // 파일이 셋 미만인 언어는 잡음으로 보고 뺀다.
        let repo = repo_with(
            "detect-repo",
            &[
                ("src/A.java", "class A {}"),
                ("src/B.java", "class B {}"),
                ("src/C.java", "class C {}"),
                ("script.py", "x = 1"),
            ],
        );
        let langs = detect_languages(&[repo]).unwrap();
        assert_eq!(langs, vec!["java"], "파이썬 한 개는 빠져야 한다");
    }

    #[test]
    fn creates_both_config_files() {
        let repo = repo_with(
            "create-repo",
            &[
                ("src/A.java", "class A {}"),
                ("src/B.java", "class B {}"),
                ("src/C.java", "class C {}"),
            ],
        );
        let dir = scratch("create-out");
        let out = init_solution(&dir, &[repo.clone()], Some("demo".into()), false).unwrap();

        assert!(out.config_path.exists(), "nunchi.toml 이 없다");
        assert!(out.shared_path.exists(), "nunchi.shared.toml 이 없다");
        assert_eq!(out.solution, "demo");
        assert_eq!(out.languages, vec!["java"]);
        assert!(!out.used_default_languages);

        // 공용 설정에는 경로가 들어가면 안 된다. 장비마다 다르기 때문이다.
        let shared = std::fs::read_to_string(&out.shared_path).unwrap();
        assert!(
            !shared.contains(repo.to_str().unwrap()),
            "공용 설정에 저장소 경로가 들어갔다:\n{shared}"
        );
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let repo = repo_with("twice-repo", &[("src/A.java", "class A {}")]);
        let dir = scratch("twice-out");
        let repos = [repo];
        init_solution(&dir, &repos, None, false).unwrap();

        let again = init_solution(&dir, &repos, None, false);
        assert!(again.is_err(), "이미 있는 설정을 덮어썼다");
        assert!(
            init_solution(&dir, &repos, None, true).is_ok(),
            "force 를 주면 덮어써야 한다"
        );
    }
}
