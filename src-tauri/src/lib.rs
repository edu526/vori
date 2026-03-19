use tauri::Manager;

mod commands;
mod models;
mod services;
mod state;

use commands::config::*;
use commands::launcher::*;
use commands::search::*;
use services::{config_manager, editor_detector, terminal};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Second launch attempt — focus the existing window instead
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Migrate config from code-launcher → vori on first launch (no-op if already done)
            config_manager::migrate_from_legacy();

            let categories = config_manager::load("categories.json").unwrap_or_default();
            let projects = config_manager::load("projects.json").unwrap_or_default();
            let files = config_manager::load("files.json").unwrap_or_default();
            let mut preferences: models::preferences::Preferences =
                config_manager::load("preferences.json").unwrap_or_default();
            let favorites = config_manager::load("favorites.json").unwrap_or_default();
            let recents = config_manager::load("recents.json").unwrap_or_default();

            // Auto-detect terminals on first launch (when none are configured yet)
            if preferences.terminal.available.is_empty() {
                preferences.terminal.available = terminal::detect_terminals();
            }
            // Auto-detect editors on first launch (when none are configured yet)
            if preferences.editors_available.is_empty() {
                preferences.editors_available = editor_detector::detect_editors();
            }
            let _ = config_manager::save("preferences.json", &preferences);

            app.manage(AppState::new(
                categories, projects, files, preferences, favorites, recents,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config — data
            get_app_data,
            // Config — categories
            add_category,
            update_category,
            delete_category,
            add_subcategory,
            update_subcategory,
            delete_subcategory,
            // Config — projects
            add_project,
            update_project,
            delete_project,
            // Config — files
            add_file,
            update_file,
            delete_file,
            // Config — preferences
            get_preferences,
            update_preferences,
            // Config — favorites
            get_favorites,
            toggle_favorite,
            // Config — recents
            get_recents,
            add_recent,
            // Launcher
            open_project_in_editor,
            open_file_in_editor,
            open_in_terminal,
            detect_terminals,
            detect_editors,
            get_installed_apps,
            // Search
            search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
