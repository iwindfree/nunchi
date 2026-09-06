// GUI 앱이므로 Windows에서 콘솔 창이 함께 뜨지 않게 한다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

/// 화면이 처음 뜰 때 보여 줄 상태.
/// nunchi-core를 실제로 부를 수 있는지 확인하는 용도이기도 하다.
#[derive(Serialize)]
struct Status {
    /// 설정 파일을 찾았는가. 없으면 init 마법사로 간다.
    config_found: bool,
    config_path: Option<String>,
    solution: Option<String>,
    repos: Vec<String>,
    /// 적용 중인 프레임워크 규칙 수
    rule_count: usize,
    /// 지원 언어 수
    language_count: usize,
}

#[tauri::command]
fn status() -> Status {
    let rules = nunchi_core::rules::FrameworkRules::effective(&Default::default());
    let rule_count = rules.route.len()
        + rules.base_path.len()
        + rules.bean.len()
        + rules.inject.len()
        + rules.http_client.len()
        + rules.persistence.len();

    let found = std::env::current_dir()
        .ok()
        .and_then(|cwd| nunchi_core::config::Config::discover(&cwd));

    match found {
        Some(path) => {
            let config = nunchi_core::config::Config::load(&path).ok();
            Status {
                config_found: config.is_some(),
                config_path: Some(path.display().to_string()),
                solution: config.as_ref().map(|c| c.solution.name.clone()),
                repos: config
                    .as_ref()
                    .map(|c| {
                        c.solution
                            .repos
                            .iter()
                            .map(|r| r.display().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                rule_count,
                language_count: rules.lang_syntax.len(),
            }
        }
        None => Status {
            config_found: false,
            config_path: None,
            solution: None,
            repos: Vec::new(),
            rule_count,
            language_count: rules.lang_syntax.len(),
        },
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![status])
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
