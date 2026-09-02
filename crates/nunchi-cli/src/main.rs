//! nunchi CLI — 단일 바이너리, 서브커맨드 (PLAN.md 용어 절)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nunchi_core::config::{Config, IndexConfig, RankWeights, Solution, CONFIG_FILE};
use nunchi_core::store::Store;
use nunchi_core::{index, lang, SqliteStore};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "nunchi",
    about = "코드베이스 컨텍스트 그래프 — 에이전트에게 답이 아니라 좌표를 준다",
    version
)]
struct Cli {
    /// nunchi.toml 경로 (기본: 상위 디렉터리에서 탐색)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// nunchi.toml 생성 — 저장소·언어 감지
    Init {
        /// 인덱싱할 저장소 경로들 (생략 시 현재 디렉터리)
        repos: Vec<PathBuf>,
        /// 솔루션 이름 (생략 시 첫 저장소 이름)
        #[arg(long)]
        name: Option<String>,
        /// 기존 설정을 덮어쓴다
        #[arg(long)]
        force: bool,
    },
    /// 인덱싱
    Index {
        /// 인덱스를 비우고 처음부터 다시 만든다
        #[arg(long)]
        rebuild: bool,
    },
    /// 인덱스 품질 검증 — 커버리지, 노드/엣지 수
    Doctor {
        /// 기계가 읽을 수 있는 JSON으로 출력 (CI 게이트용)
        #[arg(long)]
        json: bool,
    },
    /// 전문 검색 — 심볼·파일을 좌표와 함께 반환
    Find {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// MCP 서버 (stdio)
    Serve,
    /// 컨텍스트 팩 — 토큰 예산 내로 랭킹된 코드 스켈레톤
    Pack {
        task: String,
        #[arg(long, default_value_t = 4000)]
        budget: usize,
    },
    /// TUI — 그래프 탐색·팩 미리보기·가중치 튜닝
    Tui,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("NUNCHI_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Init { repos, name, force } => cmd_init(repos, name, force),
        Command::Index { rebuild } => cmd_index(cli.config, rebuild),
        Command::Doctor { json } => cmd_doctor(cli.config, json),
        Command::Find { query, limit, json } => cmd_find(cli.config, &query, limit, json),
        Command::Serve => not_yet("serve", "Phase 1 — rmcp 연동"),
        Command::Pack { .. } => not_yet("pack", "Phase 2 — 랭킹 + 토큰 예산 렌더링"),
        Command::Tui => not_yet("tui", "Phase 3.5 — ratatui"),
    }
}

fn not_yet(name: &str, phase: &str) -> Result<()> {
    anyhow::bail!("`nunchi {name}`은 아직 구현되지 않았습니다 ({phase}). PLAN.md 4절 참조.")
}

/// 설정 파일 위치와 인덱스 경로를 함께 해결한다.
/// 인덱스는 설정 파일 옆 `.nunchi/graph.db`에 둔다.
fn resolve(config_arg: Option<PathBuf>) -> Result<(Config, PathBuf)> {
    let config_path = match config_arg {
        Some(p) => p,
        None => {
            let cwd = std::env::current_dir()?;
            Config::discover(&cwd).with_context(|| {
                format!("{CONFIG_FILE}을 찾을 수 없습니다. `nunchi init`을 먼저 실행하세요.")
            })?
        }
    };
    let config = Config::load(&config_path)?;
    let base = config_path.parent().unwrap_or(Path::new("."));
    Ok((config, base.join(".nunchi").join("graph.db")))
}

fn cmd_init(repos: Vec<PathBuf>, name: Option<String>, force: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let target = cwd.join(CONFIG_FILE);
    if target.exists() && !force {
        anyhow::bail!("{CONFIG_FILE}이 이미 있습니다. 덮어쓰려면 --force를 쓰세요.");
    }

    let repos = if repos.is_empty() { vec![cwd.clone()] } else { repos };
    let mut resolved = Vec::new();
    for r in &repos {
        resolved.push(
            r.canonicalize()
                .with_context(|| format!("저장소 경로를 찾을 수 없습니다: {}", r.display()))?,
        );
    }

    let detected = detect_languages(&resolved)?;
    let solution_name = name.unwrap_or_else(|| {
        resolved[0]
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "solution".into())
    });

    let config = Config {
        solution: Solution { name: solution_name.clone(), repos: resolved.clone() },
        index: IndexConfig {
            languages: if detected.is_empty() {
                IndexConfig::default().languages
            } else {
                detected.clone()
            },
            ..IndexConfig::default()
        },
        rank: RankWeights::default(),
    };
    config.save(&target)?;

    println!("{} 생성", target.display());
    println!("  솔루션  {solution_name}");
    println!("  저장소  {}개", resolved.len());
    for r in &resolved {
        println!("          {}", r.display());
    }
    println!("  언어    {}", if detected.is_empty() { "(감지 실패 — 기본값)".into() } else { detected.join(", ") });
    println!("\n제외 패턴을 확인하세요. 생성 코드가 인덱스에 들어오면 랭킹이 오염됩니다.");
    println!("다음: nunchi index");
    Ok(())
}

/// 저장소를 훑어 실제로 존재하는 코드 언어를 찾는다.
fn detect_languages(repos: &[PathBuf]) -> Result<Vec<String>> {
    use std::collections::BTreeMap;
    let excludes = index::build_exclude_set(
        &nunchi_core::config::DEFAULT_EXCLUDES
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
            let Some(rel) = nunchi_core::path::relative_to(root, entry.path()) else {
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

fn cmd_index(config_arg: Option<PathBuf>, rebuild: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    let mut store = SqliteStore::open(&db_path)?;
    if rebuild {
        store.clear()?;
    }

    let started = std::time::Instant::now();
    let stats = index::index_all(&config, &mut store)?;
    let elapsed = started.elapsed();

    println!("인덱싱 완료  {:.2}s", elapsed.as_secs_f64());
    println!("  저장소   {}", stats.repos);
    println!("  파일     {} 인덱싱 / {} 탐색", stats.files_indexed, stats.files_seen);
    if stats.files_skipped_size > 0 || stats.files_skipped_binary > 0 {
        println!(
            "  건너뜀   크기초과 {} · 바이너리 {}",
            stats.files_skipped_size, stats.files_skipped_binary
        );
    }
    println!("  노드     {}", store.count_nodes()?);
    println!("  엣지     {}", store.count_edges()?);
    println!("  인덱스   {}", db_path.display());
    println!("\n다음: nunchi doctor");
    Ok(())
}

fn cmd_doctor(config_arg: Option<PathBuf>, json: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    if !db_path.exists() {
        anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
    }
    let store = SqliteStore::open(&db_path)?;
    let by_lang = store.files_by_lang()?;
    let nodes = store.count_nodes()?;
    let edges = store.count_edges()?;

    if json {
        let report = serde_json::json!({
            "solution": config.solution.name,
            "nodes": nodes,
            "edges": edges,
            "files_by_lang": by_lang.iter()
                .map(|(l, n)| serde_json::json!({"lang": l, "files": n}))
                .collect::<Vec<_>>(),
            // 심볼 추출이 들어오면 채워진다 (Phase 1 다음 단계).
            "symbol_resolution_rate": serde_json::Value::Null,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("solution: {}\n", config.solution.name);
    println!("언어 커버리지");
    for (l, n) in &by_lang {
        let marker = if lang::is_code(l) { "  " } else { "· " };
        println!("  {marker}{l:<14}{n:>7} files");
    }
    println!("\n인덱스     노드 {nodes} · 엣지 {edges}");
    println!("\n심볼 해소율   (미측정 — 심볼 추출기 구현 후 산출)");
    println!("스모크 테스트 (미구현)");
    println!("\n⚠ 현재는 File 노드까지만 인덱싱합니다. tree-sitter 심볼 추출이 다음 단계입니다.");
    Ok(())
}

fn cmd_find(config_arg: Option<PathBuf>, query: &str, limit: usize, json: bool) -> Result<()> {
    let (_, db_path) = resolve(config_arg)?;
    if !db_path.exists() {
        anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
    }
    let store = SqliteStore::open(&db_path)?;
    let hits = store.search(query, limit)?;

    if json {
        let out: Vec<_> = hits
            .iter()
            .map(|h| {
                serde_json::json!({
                    "ref": h.node.reference(),
                    "kind": h.node.kind.as_str(),
                    "name": h.node.name,
                    "repo": h.node.repo,
                    "lang": h.node.lang,
                    "score": h.score,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if hits.is_empty() {
        println!("결과 없음: {query}");
        return Ok(());
    }
    for h in &hits {
        println!(
            "{:>6.2}  {:<8} {}",
            h.score,
            h.node.kind.as_str(),
            h.node.reference().unwrap_or_else(|| h.node.name.clone())
        );
    }
    Ok(())
}
