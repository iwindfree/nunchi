//! 설정 편집.
//!
//! 자주 고치는 값은 폼으로 받고, 프레임워크 규칙처럼 구조가 깊은 것은 TOML
//! 원문을 그대로 고치게 한다. 폼에 모든 항목을 넣으려 하면 규칙 하나를
//! 추가할 때마다 화면을 고쳐야 한다.
//!
//! 저장할 때는 `toml_edit`으로 바꿀 키만 갈아 끼운다. 파일을 통째로 다시
//! 쓰면 손으로 적은 주석이 사라지는데, 이 프로젝트에서 주석은 "실측에서
//! 오탐이 21건 중 16건이었다" 같은 판단의 근거를 남기는 자리다.

use anyhow::{Context, Result};
use nunchi_core::config::{CONFIG_FILE, Config, RankWeights, SHARED_FILE, SharedConfig};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

/// 폼이 다루는 값들.
#[derive(Serialize, Deserialize, Clone)]
pub struct Form {
    pub name: String,
    pub languages: Vec<String>,
    pub exclude: Vec<String>,
    pub max_file_bytes: u64,
    pub max_commits: usize,
    pub max_candidates: usize,
    pub rank: RankWeights,
    pub synonyms: Vec<Synonym>,
}

/// 도메인 용어 하나와 그것이 가리키는 영어 식별자들.
#[derive(Serialize, Deserialize, Clone)]
pub struct Synonym {
    pub term: String,
    pub words: Vec<String>,
}

pub fn read(config_path: &Path) -> Result<Form> {
    let config = Config::load(config_path)?;
    let mut synonyms: Vec<Synonym> = config
        .semantic
        .terms
        .iter()
        .map(|(term, words)| Synonym {
            term: term.clone(),
            words: words.clone(),
        })
        .collect();
    // HashMap은 순서가 없으므로 화면이 열릴 때마다 줄이 뒤바뀐다.
    synonyms.sort_by(|a, b| a.term.cmp(&b.term));

    Ok(Form {
        name: config.solution.name,
        languages: config.index.languages,
        exclude: config.index.exclude,
        max_file_bytes: config.index.max_file_bytes,
        max_commits: config.index.max_commits,
        max_candidates: config.index.max_candidates,
        rank: config.rank,
        synonyms,
    })
}

/// 폼의 값을 두 파일에 나누어 쓴다.
///
/// 경로가 없는 값은 공용 파일에도 넣는다. 불러올 때 공용 파일이 나중에
/// 덮어쓰므로, 한쪽만 고치면 화면에 보이는 값과 실제로 쓰이는 값이 어긋난다.
pub fn save(config_path: &Path, form: &Form) -> Result<()> {
    if form.name.trim().is_empty() {
        anyhow::bail!("솔루션 이름을 비워 둘 수 없습니다.");
    }
    if form.languages.is_empty() {
        anyhow::bail!("언어를 하나 이상 골라야 합니다.");
    }
    if form.max_candidates == 0 {
        anyhow::bail!("후보 상한은 1 이상이어야 합니다.");
    }

    let mut local = open_doc(config_path)?;
    table(&mut local, "solution")?.insert("name", string(&form.name));
    let index = table(&mut local, "index")?;
    index.insert("languages", inline_array(&form.languages));
    index.insert("exclude", block_array(&form.exclude));
    index.insert("max_file_bytes", int(form.max_file_bytes as i64));
    index.insert("max_commits", int(form.max_commits as i64));
    index.insert("max_candidates", int(form.max_candidates as i64));
    write_doc(config_path, &local)?;

    let shared_path = config_path.with_file_name(SHARED_FILE);
    let mut shared = open_doc(&shared_path)?;
    let index = table(&mut shared, "index")?;
    index.insert("languages", inline_array(&form.languages));
    index.insert("exclude", block_array(&form.exclude));
    index.insert("max_commits", int(form.max_commits as i64));
    index.insert("max_candidates", int(form.max_candidates as i64));
    put_rank(&mut shared, &form.rank)?;
    put_synonyms(&mut shared, &form.synonyms)?;
    write_doc(&shared_path, &shared)?;
    Ok(())
}

/// 팩 화면에서 맞춘 가중치만 공용 파일에 넣는다.
pub fn save_weights(config_path: &Path, rank: &RankWeights) -> Result<PathBuf> {
    let path = config_path.with_file_name(SHARED_FILE);
    let mut doc = open_doc(&path)?;
    put_rank(&mut doc, rank)?;
    write_doc(&path, &doc)?;
    Ok(path)
}

/// 편집기에 띄울 TOML 원문.
#[derive(Serialize)]
pub struct RawToml {
    pub path: String,
    pub text: String,
    /// 파일이 아직 없으면 빈 내용으로 시작한다.
    pub exists: bool,
}

fn raw_path(config_path: &Path, which: &str) -> Result<PathBuf> {
    match which {
        "local" => Ok(config_path.to_path_buf()),
        "shared" => Ok(config_path.with_file_name(SHARED_FILE)),
        other => anyhow::bail!("알 수 없는 파일입니다: {other}"),
    }
}

pub fn read_raw(config_path: &Path, which: &str) -> Result<RawToml> {
    let path = raw_path(config_path, which)?;
    let exists = path.is_file();
    let text = if exists {
        std::fs::read_to_string(&path)
            .with_context(|| format!("파일을 읽을 수 없습니다: {}", path.display()))?
    } else {
        String::new()
    };
    Ok(RawToml {
        path: path.display().to_string(),
        text,
        exists,
    })
}

/// 원문을 저장하기 전에 파싱해 본다.
///
/// 깨진 TOML을 그대로 쓰면 다음에 앱을 열 때 설정을 읽지 못한다. 문법만
/// 보는 것이 아니라 실제 설정 타입으로 읽어 봐야 `max_commits = "많이"`
/// 같은 것도 걸러진다.
pub fn save_raw(config_path: &Path, which: &str, text: &str) -> Result<()> {
    let path = raw_path(config_path, which)?;
    match which {
        "local" => {
            toml::from_str::<Config>(text)
                .with_context(|| format!("{}의 내용이 설정 형식에 맞지 않습니다", CONFIG_FILE))?;
        }
        _ => {
            toml::from_str::<SharedConfig>(text).with_context(|| {
                format!("{}의 내용이 공용 설정 형식에 맞지 않습니다", SHARED_FILE)
            })?;
        }
    }
    std::fs::write(&path, text)
        .with_context(|| format!("파일을 쓸 수 없습니다: {}", path.display()))?;
    Ok(())
}

// ── toml_edit 다루기 ─────────────────────────────────────

fn open_doc(path: &Path) -> Result<DocumentMut> {
    if !path.is_file() {
        return Ok(DocumentMut::new());
    }
    std::fs::read_to_string(path)
        .with_context(|| format!("파일을 읽을 수 없습니다: {}", path.display()))?
        .parse::<DocumentMut>()
        .with_context(|| format!("TOML을 파싱하지 못했습니다: {}", path.display()))
}

fn write_doc(path: &Path, doc: &DocumentMut) -> Result<()> {
    std::fs::write(path, doc.to_string())
        .with_context(|| format!("파일을 쓸 수 없습니다: {}", path.display()))
}

/// 표가 없으면 만들고, 있으면 그대로 돌려준다.
fn table<'a>(doc: &'a mut DocumentMut, key: &str) -> Result<&'a mut Table> {
    doc.as_table_mut()
        .entry(key)
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .with_context(|| format!("[{key}] 자리에 표가 아닌 값이 들어 있습니다."))
}

fn string(s: &str) -> Item {
    Item::Value(Value::from(s))
}

fn int(n: i64) -> Item {
    Item::Value(Value::from(n))
}

/// 짧은 목록. 한 줄로 쓴다.
fn inline_array(items: &[String]) -> Item {
    let mut arr = Array::new();
    for s in items {
        arr.push(s.as_str());
    }
    Item::Value(Value::Array(arr))
}

/// 긴 목록. 한 줄에 하나씩 쓴다. 제외 패턴은 스무 개를 넘기도 한다.
fn block_array(items: &[String]) -> Item {
    let mut arr = Array::new();
    for s in items {
        let mut v = Value::from(s.as_str());
        v.decor_mut().set_prefix("\n  ");
        arr.push_formatted(v);
    }
    if !items.is_empty() {
        arr.set_trailing_comma(true);
        arr.set_trailing("\n");
    }
    Item::Value(Value::Array(arr))
}

fn put_rank(doc: &mut DocumentMut, rank: &RankWeights) -> Result<()> {
    let t = table(doc, "rank")?;
    for (key, value) in [
        ("alpha_bm25", rank.alpha_bm25),
        ("beta_ppr", rank.beta_ppr),
        ("gamma_recency", rank.gamma_recency),
        ("delta_cochange", rank.delta_cochange),
        ("epsilon_central", rank.epsilon_central),
    ] {
        // 소수점 둘째 자리까지만 쓴다. 슬라이더가 만드는 0.30000001을 그대로
        // 남기면 파일이 지저분해진다.
        let rounded = (value * 100.0).round() / 100.0;
        t.insert(key, Item::Value(Value::from(rounded as f64)));
    }
    Ok(())
}

fn put_synonyms(doc: &mut DocumentMut, list: &[Synonym]) -> Result<()> {
    let mut terms = Table::new();
    for s in list {
        let term = s.term.trim();
        let words: Vec<String> = s
            .words
            .iter()
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();
        if term.is_empty() || words.is_empty() {
            continue;
        }
        terms.insert(term, inline_array(&words));
    }
    table(doc, "semantic")?.insert("terms", Item::Table(terms));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> Form {
        Form {
            name: "demo".into(),
            languages: vec!["java".into(), "typescript".into()],
            exclude: vec!["**/target".into()],
            max_file_bytes: 2 * 1024 * 1024,
            max_commits: 500,
            max_candidates: 5,
            rank: RankWeights::default(),
            synonyms: vec![Synonym {
                term: "댓글".into(),
                words: vec!["comment".into()],
            }],
        }
    }

    /// 폼에 없는 항목과 주석이 저장 뒤에도 남아 있어야 한다.
    #[test]
    fn keeps_comments_and_unknown_keys() {
        let dir = std::env::temp_dir().join(format!("nunchi-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CONFIG_FILE);
        std::fs::write(
            &path,
            "# 손으로 적은 메모\n[solution]\nname = \"old\"\nrepos = [\"/tmp/a\"]\n",
        )
        .unwrap();

        save(&path, &form()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# 손으로 적은 메모"));
        assert!(text.contains("/tmp/a"));
        assert!(text.contains("name = \"demo\""));

        let shared = std::fs::read_to_string(dir.join(SHARED_FILE)).unwrap();
        assert!(shared.contains("max_candidates = 5"));
        assert!(shared.contains("\"댓글\""));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 깨진 원문은 쓰지 않고 되돌려야 한다.
    #[test]
    fn rejects_broken_toml() {
        let dir = std::env::temp_dir().join(format!("nunchi-raw-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CONFIG_FILE);
        std::fs::write(&path, "[solution]\nname = \"a\"\nrepos = []\n").unwrap();

        assert!(save_raw(&path, "local", "[solution\nname =").is_err());
        assert!(save_raw(&path, "shared", "[rank]\nalpha_bm25 = \"높게\"\n").is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[solution]\nname = \"a\"\nrepos = []\n"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
