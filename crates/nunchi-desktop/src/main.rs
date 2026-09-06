// GUI 앱이므로 Windows에서 콘솔 창이 함께 뜨지 않게 한다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::Overview;

/// 설정과 인덱스의 현재 상태를 한 번에 돌려준다.
/// 화면이 처음 뜰 때와 인덱싱이 끝난 뒤에 부른다.
#[tauri::command]
fn overview() -> Overview {
    state::overview()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![overview])
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
