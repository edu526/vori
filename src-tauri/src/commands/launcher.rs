use tauri::State;

use crate::services::{config_manager, editor, terminal};
use crate::state::AppState;

#[tauri::command]
pub fn open_project_in_editor(
    path: String,
    editor_name: String,
    state: State<AppState>,
) -> Result<(), String> {
    editor::open_in_editor(&path, &editor_name)?;
    // If close_on_open, the frontend handles window close after this returns Ok
    let _ = state.preferences.lock().unwrap().close_on_open;
    Ok(())
}

#[tauri::command]
pub fn open_file_in_editor(path: String, text_editor: Option<String>) -> Result<(), String> {
    editor::open_file_in_text_editor(&path, text_editor.as_deref())
}

#[tauri::command]
pub fn open_in_terminal(path: Option<String>, state: State<AppState>) -> Result<(), String> {
    let terminal_cmd = {
        let prefs = state.preferences.lock().unwrap();
        let preferred = prefs
            .terminal
            .preferred
            .clone()
            .unwrap_or_else(|| "xterm".to_string());
        // Resolve name → full binary path from the detected terminals map.
        // Falls back to the name itself so plain commands like "xterm" still work.
        prefs
            .terminal
            .available
            .get(&preferred)
            .cloned()
            .unwrap_or(preferred)
    };
    terminal::open_terminal(path.as_deref(), &terminal_cmd)
}

#[tauri::command]
pub fn detect_terminals(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let found = terminal::detect_terminals();
    {
        let mut prefs = state.preferences.lock().unwrap();
        prefs.terminal.available = found.clone();
        prefs.terminal.last_detected = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );
        config_manager::save("preferences.json", &*prefs)?;
    }
    Ok(found)
}
