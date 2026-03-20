#![allow(unused)]
use tauri::Manager;
pub fn test(app: &tauri::AppHandle) {
    let _ = app.remove_tray_by_id("main");
}
