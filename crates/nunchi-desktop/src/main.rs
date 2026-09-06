// GUI 앱이므로 Windows에서 콘솔 창이 함께 뜨지 않게 한다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use serde::Serialize;
use state::Overview;
use std::path::PathBuf;

#[tauri::command]
fn overview() -> Overview {
    state::overview()
}

/// 네이티브 폴더 선택 대화상자를 연다.
///
/// 이 앱을 만든 가장 큰 이유다. 브라우저는 보안상 로컬 절대 경로를
/// 자바스크립트에 넘겨주지 않으므로, 웹으로 만들면 저장소 경로를 손으로
/// 입력해야 한다.
///
/// 취소하면 `None`이다.
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    // 대화상자는 블로킹으로 열어야 하므로 별도 스레드에서 기다린다.
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

/// 고른 저장소들을 훑어 어떤 언어가 있는지 미리 보여 준다.
/// 설정을 만들기 전에 확인할 수 있게 하려는 것이다.
#[tauri::command]
fn detect_languages(repos: Vec<String>) -> Result<Vec<String>, String> {
    let paths: Vec<PathBuf> = repos.into_iter().map(PathBuf::from).collect();
    nunchi_core::init::detect_languages(&paths).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct InitResult {
    config_path: String,
    shared_path: String,
    solution: String,
    languages: Vec<String>,
    used_default_languages: bool,
}

/// 설정 파일 두 개를 만든다. CLI의 `nunchi init`과 같은 함수를 쓴다.
#[tauri::command]
fn init_solution(
    dir: String,
    repos: Vec<String>,
    name: Option<String>,
    force: bool,
) -> Result<InitResult, String> {
    let paths: Vec<PathBuf> = repos.into_iter().map(PathBuf::from).collect();
    let name = name.filter(|n| !n.trim().is_empty());
    let out = nunchi_core::init::init_solution(std::path::Path::new(&dir), &paths, name, force)
        .map_err(|e| e.to_string())?;
    Ok(InitResult {
        config_path: out.config_path.display().to_string(),
        shared_path: out.shared_path.display().to_string(),
        solution: out.solution,
        languages: out.languages,
        used_default_languages: out.used_default_languages,
    })
}

/// 설정을 어디에 만들지 정하는 기본값. 앱을 실행한 디렉터리다.
#[tauri::command]
fn default_dir() -> String {
    std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            overview,
            pick_folder,
            detect_languages,
            init_solution,
            default_dir
        ])
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
