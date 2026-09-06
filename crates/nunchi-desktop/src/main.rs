// GUI 앱이므로 Windows에서 콘솔 창이 함께 뜨지 않게 한다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod recent;
mod state;

use serde::Serialize;
use state::Overview;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

/// 지금 열려 있는 솔루션의 설정 파일 경로.
///
/// 데스크톱 앱은 어디서 실행될지 알 수 없으므로 CLI처럼 현재 디렉터리에
/// 기대지 않는다. 무엇을 열었는지 앱이 들고 있다가 각 커맨드에 넘긴다.
#[derive(Default)]
struct Opened(Mutex<Option<PathBuf>>);

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
    Ok(view)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Opened::default())
        .invoke_handler(tauri::generate_handler![
            startup,
            open_solution,
            overview,
            recent_list,
            forget_solution,
            pick_folder,
            open_folder,
            detect_languages,
            init_solution
        ])
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
