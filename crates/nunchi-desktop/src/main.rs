// GUI 앱이므로 Windows에서 콘솔 창이 함께 뜨지 않게 한다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod query;
mod recent;
mod settings;
mod state;

use serde::Serialize;
use state::Overview;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

/// 지금 열려 있는 솔루션의 설정 파일 경로.
///
/// 데스크톱 앱은 어디서 실행될지 알 수 없으므로 CLI처럼 현재 디렉터리에
/// 기대지 않는다. 무엇을 열었는지 앱이 들고 있다가 각 커맨드에 넘긴다.
#[derive(Default)]
struct Opened(Mutex<Option<PathBuf>>);

/// 인덱싱이 도는 중인지. 겹쳐 실행하면 같은 데이터베이스를 두 곳에서 쓰게 된다.
#[derive(Default)]
struct Indexing(std::sync::atomic::AtomicBool);

/// 열어 둔 인덱스. 탐색과 팩이 같은 그래프를 다시 쓴다.
#[derive(Default)]
struct Session(Mutex<Option<query::Session>>);

/// 들고 있던 인덱스를 버린다. 인덱싱을 다시 했거나 설정이 바뀐 뒤에 부른다.
fn drop_session(app: &tauri::AppHandle) {
    if let Ok(mut slot) = app.state::<Session>().0.lock() {
        *slot = None;
    }
}

/// 지금 열린 솔루션의 인덱스를 넘겨 준다. 아직 없으면 그때 연다.
fn with_session<T>(
    app: &tauri::AppHandle,
    f: impl FnOnce(&mut query::Session) -> Result<T, String>,
) -> Result<T, String> {
    let Some(config_path) = current(app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    let state = app.state::<Session>();
    let mut slot = state.0.lock().map_err(|_| "상태를 읽지 못했습니다.".to_string())?;
    // 다른 솔루션을 열었으면 들고 있던 것을 버리고 새로 연다.
    if slot.as_ref().is_none_or(|s| s.config_path != config_path) {
        *slot = Some(query::Session::open(&config_path).map_err(|e| e.to_string())?);
    }
    f(slot.as_mut().expect("바로 위에서 채웠다"))
}

/// 앱 데이터 디렉터리. 최근 목록을 여기 둔다.
fn app_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("nunchi"))
}

fn current(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.state::<Opened>().0.lock().ok()?.clone()
}

/// 앱을 열었을 때 무엇을 보여 줄지 정하는 데 필요한 정보.
#[derive(Serialize)]
struct Startup {
    /// 최근에 연 솔루션들. 최신 순이다.
    recent: Vec<recent::Entry>,
    /// 마지막에 열었던 솔루션을 바로 열 수 있으면 그 개요를 함께 보낸다.
    opened: Option<Overview>,
    opened_path: Option<String>,
}

#[tauri::command]
fn startup(app: tauri::AppHandle) -> Startup {
    let dir = app_dir(&app);
    let list = recent::load(&dir);
    // 가장 최근에 열었고 아직 파일이 있는 것을 자동으로 연다.
    let first = list.iter().find(|e| e.exists).cloned();
    match first {
        Some(entry) => {
            let path = PathBuf::from(&entry.config_path);
            let view = state::overview(&path);
            *app.state::<Opened>().0.lock().unwrap() = Some(path);
            Startup {
                recent: list,
                opened: Some(view),
                opened_path: Some(entry.config_path),
            }
        }
        None => Startup {
            recent: list,
            opened: None,
            opened_path: None,
        },
    }
}

/// 목록에서 고른 솔루션을 연다.
#[tauri::command]
fn open_solution(app: tauri::AppHandle, config_path: String) -> Result<Overview, String> {
    let path = PathBuf::from(&config_path);
    if !path.is_file() {
        return Err(format!("{config_path} 파일이 없습니다."));
    }
    let view = state::overview(&path);
    if let Some(c) = &view.config {
        let _ = recent::touch(&app_dir(&app), &path, &c.solution);
    }
    *app.state::<Opened>().0.lock().unwrap() = Some(path);
    drop_session(&app);
    Ok(view)
}

/// 지금 열린 솔루션을 다시 읽는다. 인덱싱이 끝난 뒤 화면을 갱신할 때 쓴다.
#[tauri::command]
fn overview(app: tauri::AppHandle) -> Option<Overview> {
    current(&app).map(|p| state::overview(&p))
}

#[tauri::command]
fn recent_list(app: tauri::AppHandle) -> Vec<recent::Entry> {
    recent::load(&app_dir(&app))
}

/// 목록에서만 지운다. 설정 파일은 그대로 둔다.
#[tauri::command]
fn forget_solution(app: tauri::AppHandle, config_path: String) -> Vec<recent::Entry> {
    recent::remove(&app_dir(&app), &config_path).unwrap_or_default()
}

/// 네이티브 폴더 선택 대화상자를 연다.
///
/// 이 앱을 만든 가장 큰 이유다. 브라우저는 보안상 로컬 절대 경로를
/// 자바스크립트에 넘겨주지 않으므로, 웹으로 만들면 저장소 경로를 손으로
/// 입력해야 한다.
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |picked| {
        let _ = tx.send(picked);
    });
    rx.recv()
        .ok()
        .flatten()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string())
}

/// 이미 설정이 있는 폴더를 골라 그 솔루션을 연다.
#[tauri::command]
async fn open_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let Some(dir) = pick_folder(app.clone()).await else {
        return Ok(None);
    };
    let path = Path::new(&dir).join(nunchi_core::config::CONFIG_FILE);
    if !path.is_file() {
        return Err(format!(
            "{dir} 에 {} 이 없습니다. 새 솔루션을 만들려면 아래 버튼을 쓰십시오.",
            nunchi_core::config::CONFIG_FILE
        ));
    }
    Ok(Some(path.display().to_string()))
}

/// 고른 저장소들을 훑어 어떤 언어가 있는지 미리 보여 준다.
#[tauri::command]
fn detect_languages(repos: Vec<String>) -> Result<Vec<String>, String> {
    let paths: Vec<PathBuf> = repos.into_iter().map(PathBuf::from).collect();
    nunchi_core::init::detect_languages(&paths).map_err(|e| e.to_string())
}

/// 설정 파일 두 개를 만들고 그 솔루션을 연다. CLI의 `nunchi init`과 같은 함수를 쓴다.
#[tauri::command]
fn init_solution(
    app: tauri::AppHandle,
    dir: String,
    repos: Vec<String>,
    name: Option<String>,
    force: bool,
) -> Result<Overview, String> {
    let paths: Vec<PathBuf> = repos.into_iter().map(PathBuf::from).collect();
    let name = name.filter(|n| !n.trim().is_empty());
    let out = nunchi_core::init::init_solution(Path::new(&dir), &paths, name, force)
        .map_err(|e| e.to_string())?;
    let _ = recent::touch(&app_dir(&app), &out.config_path, &out.solution);
    let view = state::overview(&out.config_path);
    *app.state::<Opened>().0.lock().unwrap() = Some(out.config_path);
    drop_session(&app);
    Ok(view)
}

/// 인덱싱을 시작한다. 오래 걸리므로 별도 스레드에서 돌리고 끝나면 알린다.
///
/// 진행 상황은 `index-progress` 이벤트로, 결과는 `index-done` 이벤트로 보낸다.
/// 화면을 멈추지 않으려면 이 방식이 필요하다.
#[tauri::command]
fn start_index(app: tauri::AppHandle, rebuild: bool) -> Result<(), String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    // 이미 돌고 있으면 겹쳐 실행하지 않는다.
    let running = app.state::<Indexing>();
    if running.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("이미 인덱싱 중입니다.".into());
    }

    std::thread::spawn(move || {
        let result = run_index(&app, &config_path, rebuild);
        // 그래프가 통째로 바뀌었으므로 들고 있던 것을 버린다.
        drop_session(&app);
        app.state::<Indexing>()
            .0
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let payload = match result {
            Ok(stats) => serde_json::json!({ "ok": true, "stats": stats }),
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
        };
        let _ = app.emit("index-done", payload);
    });
    Ok(())
}

fn run_index(
    app: &tauri::AppHandle,
    config_path: &Path,
    rebuild: bool,
) -> anyhow::Result<nunchi_core::index::IndexStats> {
    let config = nunchi_core::config::Config::load(config_path)?;
    let dir = config_path.parent().unwrap_or(Path::new("."));
    let db_path = dir.join(".nunchi").join("graph.db");
    let cache_path = db_path.with_file_name("extract-cache.db");

    if rebuild {
        // 스키마 버전이 바뀌면 open 이 먼저 실패하므로 파일부터 지운다.
        for suffix in ["", "-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{suffix}", db_path.display()));
            let _ = std::fs::remove_file(p);
        }
    }

    let mut store = nunchi_core::store::sqlite::SqliteStore::open(&db_path)?;
    let mut cache = nunchi_core::cache::ExtractCache::open(&cache_path)?;
    let handle = app.clone();
    let stats = nunchi_core::index::index_all_with_progress(
        &config,
        &mut store,
        Some(&mut cache),
        &mut |p| {
            let _ = handle.emit("index-progress", &p);
        },
    )?;
    Ok(stats)
}

// ── 탐색과 팩 ────────────────────────────────────────────

/// 심볼을 찾는다. 도메인 용어 사전이 질의를 넓혀 준다.
#[tauri::command]
fn search(app: tauri::AppHandle, query: String, limit: usize) -> Result<Vec<query::Hit>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    with_session(&app, |s| s.search(&query, limit).map_err(|e| e.to_string()))
}

/// 고른 심볼과 이어진 것들. 무엇이 함께 바뀌어야 하는지 본다.
#[tauri::command]
fn neighbors(app: tauri::AppHandle, id: String, depth: u32) -> Result<Vec<query::Hit>, String> {
    with_session(&app, |s| s.neighbors(&id, depth).map_err(|e| e.to_string()))
}

/// 팩 화면을 열 때 슬라이더에 채울 값.
#[tauri::command]
fn pack_defaults(app: tauri::AppHandle) -> Result<nunchi_core::config::RankWeights, String> {
    with_session(&app, |s| Ok(s.weights()))
}

#[tauri::command]
fn build_pack(
    app: tauri::AppHandle,
    task: String,
    budget: usize,
    weights: nunchi_core::config::RankWeights,
) -> Result<query::PackView, String> {
    if task.trim().is_empty() {
        return Err("무엇을 하려는지 한 문장으로 적어 주십시오.".into());
    }
    with_session(&app, |s| {
        s.pack(&task, budget, weights).map_err(|e| e.to_string())
    })
}

/// 팩 화면에서 맞춘 가중치를 공용 설정에 저장한다.
#[tauri::command]
fn save_weights(
    app: tauri::AppHandle,
    weights: nunchi_core::config::RankWeights,
) -> Result<String, String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    let path = settings::save_weights(&config_path, &weights).map_err(|e| e.to_string())?;
    // 저장한 값으로 다시 읽게 한다.
    drop_session(&app);
    Ok(path.display().to_string())
}

// ── 설정 편집 ────────────────────────────────────────────

#[tauri::command]
fn settings_read(app: tauri::AppHandle) -> Result<settings::Form, String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    settings::read(&config_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn settings_save(app: tauri::AppHandle, form: settings::Form) -> Result<Overview, String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    settings::save(&config_path, &form).map_err(|e| e.to_string())?;
    drop_session(&app);
    Ok(state::overview(&config_path))
}

/// TOML 원문을 그대로 읽는다. 폼에 없는 항목은 여기서 고친다.
#[tauri::command]
fn read_toml(app: tauri::AppHandle, which: String) -> Result<settings::RawToml, String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    settings::read_raw(&config_path, &which).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_toml(app: tauri::AppHandle, which: String, text: String) -> Result<Overview, String> {
    let Some(config_path) = current(&app) else {
        return Err("먼저 솔루션을 여십시오.".into());
    };
    settings::save_raw(&config_path, &which, &text).map_err(|e| e.to_string())?;
    drop_session(&app);
    Ok(state::overview(&config_path))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Opened::default())
        .manage(Indexing::default())
        .manage(Session::default())
        .invoke_handler(tauri::generate_handler![
            startup,
            open_solution,
            overview,
            recent_list,
            forget_solution,
            pick_folder,
            open_folder,
            detect_languages,
            init_solution,
            start_index,
            search,
            neighbors,
            pack_defaults,
            build_pack,
            save_weights,
            settings_read,
            settings_save,
            read_toml,
            save_toml
        ])
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
