use tauri::State;

use crate::services::stack_detector;
use crate::models::{
    category::{CategoriesMap, Category},
    favorites::Favorites,
    file_entry::{FileEntry, FilesMap},
    preferences::Preferences,
    project::{Project, ProjectsMap},
    recents::{RecentItem, RecentsList, MAX_RECENTS},
};
use crate::services::config_manager;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct AppData {
    pub categories: CategoriesMap,
    pub projects: ProjectsMap,
    pub files: FilesMap,
    pub preferences: Preferences,
    pub favorites: Favorites,
    pub recents: RecentsList,
    /// Config files reset at startup because they could not be parsed.
    pub recovery_notes: Vec<String>,
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn get_app_data(state: State<AppState>) -> Result<AppData, String> {
    Ok(AppData {
        categories: state.categories.lock().unwrap().clone(),
        projects: state.projects.lock().unwrap().clone(),
        files: state.files.lock().unwrap().clone(),
        preferences: state.preferences.lock().unwrap().clone(),
        favorites: state.favorites.lock().unwrap().clone(),
        recents: state.recents.lock().unwrap().clone(),
        recovery_notes: config_manager::recovery_notes(),
    })
}

// ── Categories ────────────────────────────────────────────────────────────────

/// Collect all descendant keys of a category (including itself), for cascading deletes.
fn collect_descendants(cats: &CategoriesMap, root: &str) -> Vec<String> {
    let mut result = vec![root.to_string()];
    for (key, cat) in cats.iter() {
        if cat.parent.as_deref() == Some(root) {
            result.extend(collect_descendants(cats, key));
        }
    }
    result
}

#[tauri::command]
pub fn add_category(key: String, parent: Option<String>, source_path: Option<String>, state: State<AppState>) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    cats.insert(key, Category { parent, source_path, claude_profile: None });
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn update_category(key: String, parent: Option<String>, source_path: Option<String>, state: State<AppState>) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    if !cats.contains_key(&key) {
        return Err(format!("Category '{key}' not found"));
    }
    // Prevent setting a descendant as parent (would create cycle)
    if let Some(ref p) = parent {
        let descendants = collect_descendants(&cats, &key);
        if descendants.contains(p) {
            return Err("Cannot set a descendant as parent (circular reference)".to_string());
        }
    }
    // Structural updates never touch the Claude profile — see `set_claude_profile`.
    let claude_profile = cats.get(&key).and_then(|c| c.claude_profile.clone());
    cats.insert(key, Category { parent, source_path, claude_profile });
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn delete_category(key: String, state: State<AppState>) -> Result<(), String> {
    let to_delete = {
        let cats = state.categories.lock().unwrap();
        collect_descendants(&*cats, &key)
    };
    {
        let mut cats = state.categories.lock().unwrap();
        for k in &to_delete { cats.remove(k); }
        config_manager::save("categories.json", &*cats)?;
    }
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|_, p| !to_delete.contains(&p.parent));
    config_manager::save("projects.json", &*projs)
}

// ── Projects ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_project(key: String, mut project: Project, state: State<AppState>) -> Result<(), String> {
    if project.stack.is_none() {
        let stack = stack_detector::detect_stack(std::path::Path::new(&project.path));
        if stack != "unknown" {
            project.stack = Some(stack);
        }
    }
    let mut projs = state.projects.lock().unwrap();
    projs.insert(key, project);
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn bulk_import_projects(
    entries: Vec<(String, Project)>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    for (key, mut project) in entries {
        if project.stack.is_none() {
            let stack = stack_detector::detect_stack(std::path::Path::new(&project.path));
            if stack != "unknown" {
                project.stack = Some(stack);
            }
        }
        projs.insert(key, project);
    }
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn update_project(
    key: String,
    mut project: Project,
    state: State<AppState>,
) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    // Structural updates never touch the Claude profile — see `set_claude_profile`.
    project.claude_profile = projs.get(&key).and_then(|p| p.claude_profile.clone());
    projs.insert(key, project);
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn delete_project(key: String, state: State<AppState>) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    projs.remove(&key);
    config_manager::save("projects.json", &*projs)
}

// ── Claude profiles ───────────────────────────────────────────────────────────

/// Assign (or clear, with `None`) the Claude profile of a category or project.
#[tauri::command]
pub fn set_claude_profile(
    target: String,
    key: String,
    profile: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let profile = profile.filter(|p| !p.is_empty());
    if let Some(name) = &profile {
        if !state.preferences.lock().unwrap().claude_profiles.contains_key(name) {
            return Err(format!("Claude profile '{name}' does not exist"));
        }
    }
    match target.as_str() {
        "category" => {
            let mut cats = state.categories.lock().unwrap();
            cats.get_mut(&key)
                .ok_or_else(|| format!("Category '{key}' not found"))?
                .claude_profile = profile;
            config_manager::save("categories.json", &*cats)
        }
        "project" => {
            let mut projs = state.projects.lock().unwrap();
            projs
                .get_mut(&key)
                .ok_or_else(|| format!("Project '{key}' not found"))?
                .claude_profile = profile;
            config_manager::save("projects.json", &*projs)
        }
        other => Err(format!("Unknown target: {other}")),
    }
}

/// Drop assignments that point at profiles that no longer exist.
fn clear_dangling_profile_refs(state: &AppState, profiles: &std::collections::HashMap<String, String>) -> Result<(), String> {
    let dangling = |p: &Option<String>| p.as_ref().is_some_and(|n| !profiles.contains_key(n));
    {
        let mut cats = state.categories.lock().unwrap();
        if cats.values().any(|c| dangling(&c.claude_profile)) {
            for c in cats.values_mut().filter(|c| dangling(&c.claude_profile)) {
                c.claude_profile = None;
            }
            config_manager::save("categories.json", &*cats)?;
        }
    }
    let mut projs = state.projects.lock().unwrap();
    if projs.values().any(|p| dangling(&p.claude_profile)) {
        for p in projs.values_mut().filter(|p| dangling(&p.claude_profile)) {
            p.claude_profile = None;
        }
        config_manager::save("projects.json", &*projs)?;
    }
    Ok(())
}

// ── Files ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_file(key: String, file: FileEntry, state: State<AppState>) -> Result<(), String> {
    let mut files = state.files.lock().unwrap();
    files.insert(key, file);
    config_manager::save("files.json", &*files)
}

#[tauri::command]
pub fn update_file(key: String, file: FileEntry, state: State<AppState>) -> Result<(), String> {
    let mut files = state.files.lock().unwrap();
    files.insert(key, file);
    config_manager::save("files.json", &*files)
}

#[tauri::command]
pub fn delete_file(key: String, state: State<AppState>) -> Result<(), String> {
    let mut files = state.files.lock().unwrap();
    files.remove(&key);
    config_manager::save("files.json", &*files)
}

// ── Preferences ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_preferences(state: State<AppState>) -> Preferences {
    state.preferences.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_preferences(
    app: tauri::AppHandle,
    prefs: Preferences,
    state: State<AppState>,
) -> Result<(), String> {
    let old_hotkey = state.preferences.lock().unwrap().hotkey.clone();
    let old_show_tray = state.preferences.lock().unwrap().show_tray;

    if prefs.show_tray != old_show_tray {
        if prefs.show_tray {
            let _ = crate::services::window::setup_tray(&app);
        } else {
            crate::services::window::remove_tray(&app);
        }
    }

    // Apply autostart change
    {
        use tauri_plugin_autostart::ManagerExt;
        if prefs.autostart {
            let _ = app.autolaunch().enable();
        } else {
            let _ = app.autolaunch().disable();
        }
    }

    // Re-register hotkey if changed
    if prefs.hotkey != old_hotkey && !prefs.hotkey.is_empty() {
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
        let _ = app.global_shortcut().unregister_all();
        app.global_shortcut()
            .on_shortcut(prefs.hotkey.as_str(), |app, _, event| {
                if event.state() == ShortcutState::Pressed {
                    crate::services::window::toggle(app);
                }
            })
            .map_err(|e| e.to_string())?;
    }

    *state.preferences.lock().unwrap() = prefs.clone();
    config_manager::save("preferences.json", &prefs)?;
    clear_dangling_profile_refs(&state, &prefs.claude_profiles)
}

// ── Workspace selection (Ctrl+click selection that survives across restarts) ──

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct WorkspaceSelectionEntry {
    pub path: String,
    pub label: String,
}

#[tauri::command]
pub fn get_workspace_selection() -> Vec<WorkspaceSelectionEntry> {
    config_manager::load("workspace_selection.json").unwrap_or_default()
}

#[tauri::command]
pub fn set_workspace_selection(entries: Vec<WorkspaceSelectionEntry>) -> Result<(), String> {
    config_manager::save("workspace_selection.json", &entries)
}

// ── Favorites ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_favorites(state: State<AppState>) -> Favorites {
    state.favorites.lock().unwrap().clone()
}

#[tauri::command]
pub fn toggle_favorite(
    key: String,
    item_type: String,
    state: State<AppState>,
) -> Result<Favorites, String> {
    let mut favs = state.favorites.lock().unwrap();
    let list = match item_type.as_str() {
        "project" => &mut favs.projects,
        "file" => &mut favs.files,
        "category" => &mut favs.categories,
        other => return Err(format!("Unknown favorite type: {other}")),
    };
    if let Some(pos) = list.iter().position(|k| k == &key) {
        list.remove(pos);
    } else {
        list.push(key);
    }
    let result = favs.clone();
    config_manager::save("favorites.json", &*favs)?;
    Ok(result)
}

// ── Recents ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_recents(state: State<AppState>) -> RecentsList {
    state.recents.lock().unwrap().clone()
}

pub fn push_recent(state: &AppState, item: RecentItem) -> Result<(), String> {
    let mut recents = state.recents.lock().unwrap();
    recents.retain(|r| r.path != item.path);
    recents.insert(0, item);
    recents.truncate(MAX_RECENTS);
    config_manager::save("recents.json", &*recents)
}

#[tauri::command]
pub fn add_recent(item: RecentItem, state: State<AppState>) -> Result<(), String> {
    push_recent(&state, item)
}
