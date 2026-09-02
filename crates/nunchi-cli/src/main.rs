//! nunchi CLI — 단일 바이너리, 서브커맨드 (PLAN.md 용어 절)

mod serve;
mod tui;
mod watch;

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
        /// 파일 변경을 감시하며 증분 재인덱싱한다 (데몬)
        #[arg(long)]
        watch: bool,
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
        #[arg(long)]
        json: bool,
    },
    /// 적용 중인 프레임워크 규칙을 출력 — nunchi.toml에 복사해 확장한다
    Rules {
        /// TOML로 출력 (그대로 nunchi.toml에 붙여넣을 수 있다)
        #[arg(long)]
        toml: bool,
    },
    /// TUI — 그래프 탐색·팩 미리보기·가중치 튜닝
    Tui,
}

fn main() -> Result<()> {
    // stdio MCP 서버는 stdout이 JSON-RPC 전용 채널이다.
    // 로그를 stdout에 쓰면 프로토콜이 깨진다 — 반드시 stderr로 보낸다.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
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
        Command::Index { rebuild, watch } => cmd_index(cli.config, rebuild, watch),
        Command::Doctor { json } => cmd_doctor(cli.config, json),
        Command::Find { query, limit, json } => cmd_find(cli.config, &query, limit, json),
        Command::Serve => {
            let (config, db_path) = resolve(cli.config)?;
            if !db_path.exists() {
                anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
            }
            serve::run(config, db_path)
        }
        Command::Pack { task, budget, json } => cmd_pack(cli.config, &task, budget, json),
        Command::Rules { toml: as_toml } => cmd_rules(cli.config, as_toml),
        Command::Tui => {
            let config_path = match cli.config.clone() {
                Some(p) => p,
                None => Config::discover(&std::env::current_dir()?)
                    .context("nunchi.toml을 찾을 수 없습니다")?,
            };
            let (config, db_path) = resolve(Some(config_path.clone()))?;
            if !db_path.exists() {
                anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
            }
            tui::run(config, config_path, db_path)
        }
    }
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
        // 비워두면 내장 규칙(Spring + React)이 적용된다.
        // `nunchi rules`로 현재 규칙을 확인하고 nunchi.toml에 추가해 확장한다.
        framework: Default::default(),
        semantic: Default::default(),
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
    println!("프레임워크 규칙은 내장 기본값(Spring + React)이 적용됩니다 — `nunchi rules`로 확인.");
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

fn cmd_index(config_arg: Option<PathBuf>, rebuild: bool, watch: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    // 캐시는 인덱스와 별도 파일이다 — 워크트리마다 인덱스는 달라도 캐시는 공유한다.
    let cache_path = db_path.with_file_name("extract-cache.db");
    // --rebuild는 파일부터 지운다. 스키마 버전이 바뀌었을 때 open()이 먼저
    // 실패하면 안내한 해결책이 동작하지 않는다.
    if rebuild {
        for suffix in ["", "-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{suffix}", db_path.display()));
            let _ = std::fs::remove_file(p);
        }
    }
    let mut store = SqliteStore::open(&db_path)?;

    let started = std::time::Instant::now();
    let mut cache = nunchi_core::cache::ExtractCache::open(&cache_path)?;
    let stats = index::index_all_with_cache(&config, &mut store, Some(&mut cache))?;
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
    println!("  심볼     {}", stats.symbols);
    if stats.cache_hits + stats.cache_misses > 0 {
        println!(
            "  캐시     적중 {}/{} ({:.0}%)",
            stats.cache_hits,
            stats.cache_hits + stats.cache_misses,
            cache.hit_rate() * 100.0
        );
    }
    println!("  인덱스   {}", db_path.display());

    if watch {
        println!();
        return watch::run(config, db_path, cache_path);
    }
    println!("\n다음: nunchi doctor");
    Ok(())
}

fn cmd_doctor(config_arg: Option<PathBuf>, json: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    if !db_path.exists() {
        anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
    }
    let store = SqliteStore::open(&db_path)?;
    let nodes = store.count_nodes()?;
    let edges = store.count_edges()?;
    let metrics: serde_json::Value = store
        .get_meta("metrics")?
        .and_then(|m| serde_json::from_str(&m).ok())
        .unwrap_or(serde_json::Value::Null);

    if json {
        let report = serde_json::json!({
            "solution": config.solution.name,
            "nodes": nodes,
            "edges": edges,
            "metrics": metrics,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("solution: {}\n", config.solution.name);

    println!("언어 커버리지");
    if let Some(langs) = metrics.get("by_lang").and_then(|v| v.as_array()) {
        for entry in langs {
            let lang_name = entry["lang"].as_str().unwrap_or("?");
            let files = entry["files"].as_u64().unwrap_or(0);
            let parsed = entry["parsed"].as_u64().unwrap_or(0);
            if lang::is_code(lang_name) {
                let pct = if files > 0 { parsed as f64 / files as f64 * 100.0 } else { 0.0 };
                let mark = if pct >= 99.0 { "✓" } else if pct >= 90.0 { "⚠" } else { "✗" };
                println!("  {lang_name:<14}{files:>6} files {parsed:>6} 파싱  {pct:>5.1}%  {mark}");
            } else {
                println!("· {lang_name:<14}{files:>6} files       — 파서 없음");
            }
        }
    }

    println!();
    let rate = metrics.get("call_link_rate").and_then(|v| v.as_f64());
    match rate {
        Some(r) => {
            let get = |k: &str| metrics.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
            println!("호출 연결률                    {:>5.1}%", r * 100.0);
            println!("  호출 {} — 해소 {} · 모호 {} · 미해소 {} · 후보과다 {}",
                get("calls_total"), get("calls_resolved"), get("calls_ambiguous"),
                get("calls_unresolved"), get("calls_dropped"));
            println!("  import — 내부 {} · 외부 {}", get("imports_internal"), get("imports_external"));

            // 연결률 숫자만으로는 판단할 수 없다. 미해소 이름이 외부 API면 정상이고,
            // 내부에 있어야 할 이름이면 추출기 결함이다. 사람이 눈으로 가른다.
            if let Some(top) = metrics.get("top_unresolved").and_then(|v| v.as_array()) {
                if !top.is_empty() {
                    println!("\n  미해소 호출 상위 — 외부 API면 정상, 내부 심볼이면 추출기 결함");
                    for e in top {
                        println!("    {:<28}{:>6}",
                            e["name"].as_str().unwrap_or("?"),
                            e["count"].as_u64().unwrap_or(0));
                    }
                }
            }
        }
        None => println!("호출 연결률   (미측정 — `nunchi index --rebuild`를 실행하세요)"),
    }

    // ── 프레임워크 의미론 · 교차 저장소 (Phase 1c) ──
    let get = |k: &str| metrics.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
    println!("\n프레임워크 의미론");
    println!("  라우트 {} · Bean {} · 주입 {}해소/{}미해소",
        get("routes"), get("beans"), get("injects_resolved"), get("injects_unresolved"));

    let api = get("api_calls");
    let linked = get("api_calls_linked");
    if api > 0 {
        let pct = linked as f64 / api as f64 * 100.0;
        let mark = if pct >= 70.0 { "✓" } else if pct >= 40.0 { "⚠" } else { "✗" };
        println!("\n교차 저장소 계약 (CALLS_API)  {mark}");
        println!("  프런트 API 호출 {api} — 백엔드 라우트에 연결 {linked} ({pct:.0}%)");
        let dynamic = get("api_calls_dynamic");
        if dynamic > 0 {
            println!("  동적 경로 {dynamic}건 제외 — 런타임에 조립되어 정적 분석 불가");
        }
        if let Some(unlinked) = metrics.get("unlinked_api_paths").and_then(|v| v.as_array()) {
            if !unlinked.is_empty() {
                println!("  미연결 경로 — 백엔드에 없거나 경로 표기가 어긋난 것들:");
                for u in unlinked {
                    println!("    {}", u.as_str().unwrap_or("?"));
                }
            }
        }
    } else {
        println!("\n교차 저장소 계약  프런트 API 호출이 탐지되지 않았습니다.");
        println!("  사내 HTTP 래퍼를 쓴다면 nunchi.toml에 규칙을 추가하세요 (`nunchi rules` 참조).");
    }

    println!("\n인덱스     노드 {nodes} · 엣지 {edges}");

    let _ = rate;
    println!("\n⚠ 빠른 경로(tree-sitter) 결과입니다. 이름 기반 해소이므로 연결률에는
  외부 라이브러리 호출이 분모로 포함됩니다 — 이 값에 95% 목표를 걸 수 없습니다.
  계획서의 심볼 해소율 95% 목표는 SCIP 정밀 경로(Phase 1b) 지표입니다.");
    Ok(())
}

fn cmd_pack(config_arg: Option<PathBuf>, task: &str, budget: usize, json: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    if !db_path.exists() {
        anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
    }
    let store = SqliteStore::open(&db_path)?;
    let graph = nunchi_core::graph::MemGraph::load(&store)?;
    let roots = nunchi_core::pack::repo_roots(&config);
    let opts = nunchi_core::pack::PackOptions {
        budget,
        weights: config.rank,
        synonyms: config.semantic.clone(),
        ..Default::default()
    };
    let pack = nunchi_core::pack::build_pack(&store, &graph, task, &roots, &opts)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&pack)?);
    } else {
        print!("{}", nunchi_core::pack::render_text(&pack));
    }
    Ok(())
}

fn cmd_rules(config_arg: Option<PathBuf>, as_toml: bool) -> Result<()> {
    // 설정이 없어도 내장 규칙을 볼 수 있어야 한다.
    let user = resolve(config_arg)
        .map(|(c, _)| c.framework)
        .unwrap_or_default();
    let effective = nunchi_core::rules::FrameworkRules::effective(&user);

    if as_toml {
        println!("{}", toml::to_string_pretty(&effective)?);
        return Ok(());
    }

    println!("적용 중인 프레임워크 규칙\n");
    println!("라우트 어노테이션");
    for r in &effective.route {
        let via = r.method_from_args_prefix.as_deref()
            .map(|p| format!("  (인자의 {p}* 가 우선)"))
            .unwrap_or_default();
        println!("  {:<8} @{:<22} → {}{via}", r.lang, r.annotation, r.method);
    }
    println!("\n경로 접두 어노테이션");
    for r in &effective.base_path {
        println!("  {:<8} @{}", r.lang, r.annotation);
    }
    println!("\nBean 스테레오타입");
    for r in &effective.bean {
        println!("  {:<8} {}", r.lang, r.annotations.join(", "));
    }
    println!("\n주입 판별");
    for r in &effective.inject {
        println!("  {:<8} @{} · final필드={} · 생성자파라미터={}",
            r.lang, r.annotations.join(" @"), r.final_fields, r.constructor_params);
    }
    println!("\nHTTP 클라이언트 호출");
    for r in &effective.http_client {
        let what = match (&r.callee, r.receiver_methods.is_empty()) {
            (Some(c), _) => format!("{c}(…)"),
            (None, false) => format!("_.{}(…)", r.receiver_methods.join("|")),
            _ => "?".into(),
        };
        let m = r.method.clone().unwrap_or_else(|| "(메서드명에서)".into());
        println!("  {:<8} {:<44} → {m}  url=인자{}", r.lang, what, r.url_arg);
    }
    println!("\n확장하려면 nunchi.toml에 추가하세요. 재빌드가 필요 없습니다:");
    println!(r#"
[[framework.http_client]]          # 사내 래퍼 지원 예시
lang = "typescript"
receiver_methods = ["fetchJson"]

[[framework.route]]                # 사내 어노테이션 예시
lang = "java"
annotation = "InternalEndpoint"
method = "POST"
"#);
    Ok(())
}

fn cmd_find(config_arg: Option<PathBuf>, query: &str, limit: usize, json: bool) -> Result<()> {
    let (config, db_path) = resolve(config_arg)?;
    if !db_path.exists() {
        anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
    }
    let store = SqliteStore::open(&db_path)?;
    let hits = store.search(&config.semantic.expand_query(query), limit)?;

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
