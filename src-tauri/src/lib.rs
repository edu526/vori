use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

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
            // Second launch attempt — toggle the existing window
            services::window::toggle(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
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
                categories, projects, files, preferences.clone(), favorites, recents,
            ));

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            if !is_autostart {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            // Apply autostart from preferences
            {
                use tauri_plugin_autostart::ManagerExt;
                if preferences.autostart {
                    let _ = app.autolaunch().enable();
                } else {
                    let _ = app.autolaunch().disable();
                }
            }

            // Register global shortcut from preferences
            if !preferences.hotkey.is_empty() {
                let hotkey = preferences.hotkey.clone();
                app.global_shortcut().on_shortcut(hotkey.as_str(), |app, _, event| {
                    if event.state() == ShortcutState::Pressed {
                        services::window::toggle(app);
                    }
                })?;
            }

            // Build tray icon + menu (conditionally)
            if preferences.show_tray {
                let _ = services::window::setup_tray(app.handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.app_handle().state::<AppState>();
                let prefs = state.preferences.lock().unwrap();
                if prefs.keep_background || prefs.show_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            quit_app,
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
