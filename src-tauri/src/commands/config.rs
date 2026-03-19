use tauri::State;

use crate::models::{
    category::{CategoriesMap, Category, Subcategory},
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
    })
}

// ── Categories ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_category(key: String, category: Category, state: State<AppState>) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    cats.insert(key, category);
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn update_category(
    key: String,
    category: Category,
    state: State<AppState>,
) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    if !cats.contains_key(&key) {
        return Err(format!("Category '{key}' not found"));
    }
    cats.insert(key, category);
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn delete_category(key: String, state: State<AppState>) -> Result<(), String> {
    {
        let mut cats = state.categories.lock().unwrap();
        cats.remove(&key);
        config_manager::save("categories.json", &*cats)?;
    }
    // Remove associated projects (drop cats lock first to avoid deadlock)
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|_, p| p.category != key);
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn add_subcategory(
    category_key: String,
    key: String,
    subcategory: Subcategory,
    state: State<AppState>,
) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    let cat = cats
        .get_mut(&category_key)
        .ok_or_else(|| format!("Category '{category_key}' not found"))?;
    cat.subcategories.insert(key, subcategory);
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn update_subcategory(
    category_key: String,
    key: String,
    subcategory: Subcategory,
    state: State<AppState>,
) -> Result<(), String> {
    let mut cats = state.categories.lock().unwrap();
    let cat = cats
        .get_mut(&category_key)
        .ok_or_else(|| format!("Category '{category_key}' not found"))?;
    cat.subcategories.insert(key, subcategory);
    config_manager::save("categories.json", &*cats)
}

#[tauri::command]
pub fn delete_subcategory(
    category_key: String,
    key: String,
    state: State<AppState>,
) -> Result<(), String> {
    {
        let mut cats = state.categories.lock().unwrap();
        let cat = cats
            .get_mut(&category_key)
            .ok_or_else(|| format!("Category '{category_key}' not found"))?;
        cat.subcategories.remove(&key);
        config_manager::save("categories.json", &*cats)?;
    }
    // Move orphaned projects: their subcategory becomes None
    let mut projs = state.projects.lock().unwrap();
    for p in projs.values_mut() {
        if p.category == category_key && p.subcategory.as_deref() == Some(&key) {
            p.subcategory = None;
        }
    }
    config_manager::save("projects.json", &*projs)
}

// ── Projects ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_project(key: String, project: Project, state: State<AppState>) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    projs.insert(key, project);
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn update_project(
    key: String,
    project: Project,
    state: State<AppState>,
) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    projs.insert(key, project);
    config_manager::save("projects.json", &*projs)
}

#[tauri::command]
pub fn delete_project(key: String, state: State<AppState>) -> Result<(), String> {
    let mut projs = state.projects.lock().unwrap();
    projs.remove(&key);
    config_manager::save("projects.json", &*projs)
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
    config_manager::save("preferences.json", &prefs)
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

#[tauri::command]
pub fn add_recent(item: RecentItem, state: State<AppState>) -> Result<(), String> {
    let mut recents = state.recents.lock().unwrap();
    recents.retain(|r| r.path != item.path);
    recents.insert(0, item);
    recents.truncate(MAX_RECENTS);
    config_manager::save("recents.json", &*recents)
}
