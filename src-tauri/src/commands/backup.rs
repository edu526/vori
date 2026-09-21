use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::services::backup::{self, Backup};
use crate::services::config_manager;
use crate::state::AppState;

/// Newest safety copies kept in `<config dir>/backups/`.
const KEEP_SAFETY_COPIES: usize = 5;

fn snapshot(state: &AppState) -> Backup {
    Backup::new(
        state.categories.lock().unwrap().clone(),
        state.projects.lock().unwrap().clone(),
        state.files.lock().unwrap().clone(),
        state.favorites.lock().unwrap().clone(),
        state.recents.lock().unwrap().clone(),
        state.preferences.lock().unwrap().clone(),
    )
}

fn write_backup(path: &Path, backup: &Backup) -> Result<(), String> {
    let json = serde_json::to_string_pretty(backup).map_err(|e| format!("Failed to serialize: {e}"))?;
    config_manager::write_atomic(path, json.as_bytes())
}

/// Save what is there now before an import replaces it, and prune old copies.
fn write_safety_copy(state: &AppState) -> Result<String, String> {
    let dir = config_manager::config_dir()?.join("backups");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;
    let path = dir.join(format!("pre-import-{}.json", backup::now_secs()));
    write_backup(&path, &snapshot(state))?;

    let mut copies: Vec<_> = std::fs::read_dir(&dir)
        .map(|it| {
            it.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().starts_with("pre-import-"))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    copies.sort();
    while copies.len() > KEEP_SAFETY_COPIES {
        let _ = std::fs::remove_file(copies.remove(0));
    }
    Ok(path.to_string_lossy().into_owned())
}

/// Write categories, projects, files, favorites, recents and preferences to `path`.
#[tauri::command]
pub fn export_config(path: String, state: State<AppState>) -> Result<(), String> {
    write_backup(Path::new(&path), &snapshot(&state))
}

#[derive(Serialize)]
pub struct ImportSummary {
    pub categories: usize,
    pub projects: usize,
    pub files: usize,
    /// Where the data that was replaced was copied to.
    pub safety_copy: String,
}

/// Replace the current categories, projects, files, favorites and recents with the contents
/// of a backup. The previous data is copied to `<config dir>/backups/` first.
#[tauri::command]
pub fn import_config(path: String, state: State<AppState>) -> Result<ImportSummary, String> {
    let size = std::fs::metadata(&path)
        .map_err(|e| format!("Could not read {path}: {e}"))?
        .len();
    if size > backup::MAX_BACKUP_BYTES {
        return Err("That file is too large to be a Vori backup".to_string());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Could not read {path}: {e}"))?;
    let incoming = backup::parse(&text)?;

    let safety_copy = write_safety_copy(&state)?;
    let summary = ImportSummary {
        categories: incoming.categories.len(),
        projects: incoming.projects.len(),
        files: incoming.files.len(),
        safety_copy,
    };

    // Same lock order everywhere (categories → projects → files → favorites → recents → preferences).
    let mut categories = state.categories.lock().unwrap();
    let mut projects = state.projects.lock().unwrap();
    let mut files = state.files.lock().unwrap();
    let mut favorites = state.favorites.lock().unwrap();
    let mut recents = state.recents.lock().unwrap();
    let mut preferences = state.preferences.lock().unwrap();

    *categories = incoming.categories;
    *projects = incoming.projects;
    *files = incoming.files;
    *favorites = incoming.favorites;
    *recents = incoming.recents;
    if let Some(imported) = &incoming.preferences {
        backup::merge_portable_preferences(&mut preferences, imported);
    }

    config_manager::save("categories.json", &*categories)?;
    config_manager::save("projects.json", &*projects)?;
    config_manager::save("files.json", &*files)?;
    config_manager::save("favorites.json", &*favorites)?;
    config_manager::save("recents.json", &*recents)?;
    config_manager::save("preferences.json", &*preferences)?;
    Ok(summary)
}
