use tauri::Manager;
fn test(app: &tauri::AppHandle) {
    app.remove_tray_by_id("main_tray");
}
