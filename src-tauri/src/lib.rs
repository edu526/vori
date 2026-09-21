use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod commands;
mod models;
mod services;
mod state;

use commands::backup::*;
use commands::cli::*;
use commands::config::*;
use commands::files_io::*;
use commands::git::*;
use commands::launcher::*;
use commands::scanner::*;
use commands::search::*;
use services::{config_manager, editor_detector, terminal};
use state::AppState;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            // Second launch: hand its arguments (`vori .`, `vori open x`, a vori:// link) to this
            // instance. With no arguments it just toggles the window.
            let action = services::cli::parse(args.get(1..).unwrap_or(&[]), &cwd);
            dispatch(app, action, true);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Migrate config from code-launcher → vori on first launch (no-op if already done)
            config_manager::migrate_from_legacy();
            config_manager::migrate_to_flat_format();

            let categories = config_manager::load_or_recover("categories.json");
            let projects = config_manager::load_or_recover("projects.json");
            let files = config_manager::load_or_recover("files.json");
            let mut preferences: models::preferences::Preferences =
                config_manager::load_or_recover("preferences.json");
            let favorites = config_manager::load_or_recover("favorites.json");
            let recents: models::recents::RecentsList = config_manager::load_or_recover("recents.json");
            let mut usage: models::usage::UsageMap = config_manager::load_or_recover("usage.json");
            // First run with usage tracking: start from what the recents already say.
            if usage.is_empty() {
                for r in &recents {
                    models::usage::record(&mut usage, &r.path, r.timestamp);
                }
            }

            // Auto-detect terminals on first launch (when none are configured yet)
            if preferences.terminal.available.is_empty() {
                preferences.terminal.available = terminal::detect_terminals();
            }
            // Pick a default preferred terminal if the user hasn't chosen one
            if preferences.terminal.preferred.is_none() && !preferences.terminal.available.is_empty() {
                let priority: &[&str] = if cfg!(windows) {
                    &["wt", "powershell", "pwsh", "cmd"]
                } else {
                    &["warp", "gnome-terminal", "konsole", "alacritty", "kitty", "tilix", "xterm"]
                };
                let chosen = priority.iter()
                    .find(|p| preferences.terminal.available.contains_key(**p))
                    .map(|s| s.to_string())
                    .or_else(|| preferences.terminal.available.keys().next().cloned());
                preferences.terminal.preferred = chosen;
            }
            // Auto-detect editors on first launch (when none are configured yet)
            if preferences.editors_available.is_empty() {
                preferences.editors_available = editor_detector::detect_editors();
            }
            let _ = config_manager::save("preferences.json", &preferences);

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");

            app.manage(AppState::new(
                categories, projects, files, preferences.clone(), favorites, recents, usage, is_autostart,
            ));

            // Arguments this instance was launched with (`vori .`, `vori open x`, a vori:// link).
            {
                let args: Vec<String> = std::env::args().skip(1).collect();
                let cwd = std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
                dispatch(app.handle(), services::cli::parse(&args, &cwd), false);
            }

            // Make `vori://` links reach the app even when it wasn't installed through a package
            // that registers the scheme (AppImage, dev builds). macOS registers via the bundle.
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    eprintln!("[vori] Could not register the vori:// scheme: {e}");
                }
            }
            // On macOS a link arrives as an event, not as an argument.
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let handle = app.handle().clone();
                let run = move |urls: Vec<String>| {
                    for url in urls {
                        dispatch(&handle, services::cli::parse(&[url], ""), true);
                    }
                };
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    run(urls.iter().map(|u| u.to_string()).collect());
                }
                app.deep_link().on_open_url(move |event| {
                    run(event.urls().iter().map(|u| u.to_string()).collect());
                });
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

            // Show the window early. Belt-and-suspenders: `on_page_load` also
            // shows it (after the page paints, to avoid a white flash), but
            // `on_page_load` doesn't always fire reliably. Showing here too
            // ensures the window appears even in dev mode where Vite may
            // hold the load event for a while.
            if !is_autostart || !preferences.show_tray {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            // Register global shortcut from preferences.
            // Non-fatal: if the hotkey is already taken by another app, log a warning
            // and continue. The app remains usable (tray icon can toggle the window),
            // and the user can pick a different hotkey from Preferences.
            if !preferences.hotkey.is_empty() {
                let hotkey = preferences.hotkey.clone();
                if let Err(e) = app.global_shortcut().on_shortcut(hotkey.as_str(), |app, _, event| {
                    if event.state() == ShortcutState::Pressed {
                        services::window::toggle(app);
                    }
                }) {
                    eprintln!(
                        "[vori] Could not register global hotkey '{}': {}.
                         The window can still be toggled via the tray icon or by launching the app again. \
                         Pick a different hotkey in Preferences to fix this.",
                        preferences.hotkey, e
                    );
                }
            }

            // Build tray icon + menu (conditionally)
            if preferences.show_tray {
                let _ = services::window::setup_tray(app.handle());
            }

            Ok(())
        })
        .on_page_load(|window, payload| {
            // Show the window only after the page finishes loading, so the
            // webview paints with the theme background instead of a white
            // flash. Skip on --autostart launches (stay hidden in tray) —
            // UNLESS the tray is disabled, in which case the app would
            // otherwise be running invisible with no way to open it.
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let state = window.app_handle().state::<AppState>();
                let autostart = state.is_autostart.load(std::sync::atomic::Ordering::Relaxed);
                let show_tray = state.preferences.lock().unwrap().show_tray;
                if !autostart || !show_tray {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.app_handle().state::<AppState>();
                let prefs = state.preferences.lock().unwrap();
                // Only hide if the user has a way to reopen the window.
                // Hiding when `show_tray = false` would leave the app running
                // invisible (no tray, and the global hotkey may not have
                // registered if it collided with another app).
                if prefs.keep_background && prefs.show_tray {
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
            // Config — Claude profiles
            set_claude_profile,
            // Config — projects
            add_project,
            update_project,
            delete_project,
            bulk_import_projects,
            // Config — files
            add_file,
            update_file,
            delete_file,
            // Files — read/write
            read_text_file,
            write_text_file,
            write_text_file_elevated,
            // Config — preferences
            get_preferences,
            update_preferences,
            // Config — favorites
            get_favorites,
            toggle_favorite,
            // Workspace selection persistence
            get_workspace_selection,
            set_workspace_selection,
            // Config — recents
            get_recents,
            add_recent,
            // Launcher
            open_project_in_editor,
            open_workspace_in_editor,
            open_file_in_editor,
            open_in_terminal,
            list_project_scripts,
            run_project_script,
            detect_terminals,
            detect_editors,
            get_installed_apps,
            // Scanner
            scan_folder,
            detect_workspaces_in_folder,
            // Git
            get_git_info,
            git_clone,
            // Backup
            export_config,
            import_config,
            // CLI / deep links
            take_pending_requests,
            // Search
            search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
