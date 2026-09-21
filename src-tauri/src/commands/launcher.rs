use tauri::State;

use crate::services::{app_search, claude_profile, config_manager, editor, editor_detector, scripts, terminal};
use crate::state::AppState;

/// `CLAUDE_CONFIG_DIR` to export when opening `path`, from the profile of the
/// project/category that contains it (see `claude_profile::resolve_profile_name`).
fn claude_config_dir_for(path: &str, state: &AppState) -> Option<String> {
    let categories = state.categories.lock().unwrap();
    let projects = state.projects.lock().unwrap();
    let prefs = state.preferences.lock().unwrap();
    claude_profile::config_dir_for_path(path, &categories, &projects, &prefs.claude_profiles)
}

/// Open a project folder in the editor named `editor_name` (a key of the detected editors,
/// or a raw command), exporting the Claude profile that applies to `path`.
pub fn open_project(path: &str, editor_name: &str, state: &AppState) -> Result<(), String> {
    // Resolve editor key → binary path from detected editors, fall back to raw name
    let binary = {
        let prefs = state.preferences.lock().unwrap();
        let b = prefs
            .editors_available
            .get(editor_name)
            .cloned()
            .unwrap_or_else(|| editor_name.to_string());
        eprintln!(
            "[vori][launcher] open_project_in_editor path={path:?} editor_name={editor_name:?} resolved_binary={b:?}"
        );
        eprintln!(
            "[vori][launcher] editors_available={:?}",
            prefs.editors_available
        );
        b
    };
    let claude_dir = claude_config_dir_for(path, state);
    editor::open_in_editor(path, &binary, claude_dir.as_deref())
}

#[tauri::command]
pub fn open_project_in_editor(
    path: String,
    editor_name: String,
    state: State<AppState>,
) -> Result<(), String> {
    open_project(&path, &editor_name, &state)
}

#[tauri::command]
pub fn open_workspace_in_editor(
    paths: Vec<String>,
    editor_name: String,
    state: State<AppState>,
) -> Result<(), String> {
    let binary = {
        let prefs = state.preferences.lock().unwrap();
        prefs
            .editors_available
            .get(&editor_name)
            .cloned()
            .unwrap_or_else(|| editor_name.clone())
    };
    // A workspace can span several folders; the first one decides the profile.
    let claude_dir = paths.first().and_then(|p| claude_config_dir_for(p, &state));
    editor::open_workspace_in_editor(&paths, &binary, claude_dir.as_deref())
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
            .unwrap_or_else(|| {
                if cfg!(windows) { "powershell".to_string() } else { "xterm".to_string() }
            });
        eprintln!(
            "[vori][launcher] open_in_terminal path={path:?} preferred={preferred:?}"
        );
        eprintln!(
            "[vori][launcher] terminal.available={:?}",
            prefs.terminal.available
        );
        // Resolve name → full binary path from the detected terminals map.
        // Falls back to the name itself so plain commands like "xterm" still work.
        let resolved = prefs
            .terminal
            .available
            .get(&preferred)
            .cloned()
            .unwrap_or(preferred);
        eprintln!("[vori][launcher] resolved terminal binary={resolved:?}");
        resolved
    };
    let claude_dir = path.as_deref().and_then(|p| claude_config_dir_for(p, &state));
    terminal::open_terminal(path.as_deref(), &terminal_cmd, claude_dir.as_deref())
}

#[tauri::command]
pub fn detect_editors(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let found = editor_detector::detect_editors();
    {
        let mut prefs = state.preferences.lock().unwrap();
        prefs.editors_available = found.clone();
        config_manager::save("preferences.json", &*prefs)?;
    }
    Ok(found)
}

#[tauri::command]
pub fn get_installed_apps() -> Vec<app_search::InstalledApp> {
    app_search::get_installed_apps()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        category::{CategoriesMap, Category},
        favorites::Favorites,
        file_entry::FilesMap,
        preferences::Preferences,
        project::{Project, ProjectsMap},
    };

    fn state_with(claude_profiles: &[(&str, &str)]) -> AppState {
        let mut categories = CategoriesMap::new();
        categories.insert(
            "work".into(),
            Category {
                parent: None,
                source_path: Some("/code/work".into()),
                claude_profile: Some("trabajo".into()),
            },
        );
        let mut projects = ProjectsMap::new();
        projects.insert(
            "api".into(),
            Project {
                path: "/code/work/api".into(),
                parent: "work".into(),
                stack: None,
                claude_profile: None,
            },
        );
        let mut prefs = Preferences::default();
        for (name, dir) in claude_profiles {
            prefs.claude_profiles.insert(name.to_string(), dir.to_string());
        }
        AppState::new(categories, projects, FilesMap::new(), prefs, Favorites::default(), vec![], false)
    }

    #[test]
    fn launcher_resolves_dir_from_app_state() {
        let state = state_with(&[("trabajo", "/home/u/.claude-work")]);
        assert_eq!(
            claude_config_dir_for("/code/work/api", &state).as_deref(),
            Some("/home/u/.claude-work")
        );
        assert_eq!(claude_config_dir_for("/code/elsewhere", &state), None);
    }

    #[test]
    fn launcher_ignores_assignment_to_a_deleted_profile() {
        let state = state_with(&[]);
        assert_eq!(claude_config_dir_for("/code/work/api", &state), None);
    }
}

/// `package.json` scripts of the project at `path` (with the package manager to run them),
/// or `None` when it isn't a Node project.
#[tauri::command]
pub fn list_project_scripts(path: String) -> Option<scripts::ProjectScripts> {
    scripts::list(std::path::Path::new(&path))
}

#[derive(serde::Serialize)]
pub struct RunScriptResult {
    /// False when the configured terminal can't be told to run a command: it was opened in the
    /// project folder instead and the user has to paste `command`.
    pub ran: bool,
    pub command: String,
}

/// Run one of the project's scripts in the preferred terminal.
#[tauri::command]
pub fn run_project_script(
    path: String,
    script: String,
    state: State<AppState>,
) -> Result<RunScriptResult, String> {
    let command = scripts::command_for(std::path::Path::new(&path), &script)?;
    let terminal_cmd = {
        let prefs = state.preferences.lock().unwrap();
        let preferred = prefs
            .terminal
            .preferred
            .clone()
            .unwrap_or_else(|| if cfg!(windows) { "powershell".to_string() } else { "xterm".to_string() });
        prefs.terminal.available.get(&preferred).cloned().unwrap_or(preferred)
    };
    let claude_dir = claude_config_dir_for(&path, &state);
    let ran = terminal::run_in_terminal(&path, &terminal_cmd, &command, claude_dir.as_deref())?;
    Ok(RunScriptResult { ran, command })
}
