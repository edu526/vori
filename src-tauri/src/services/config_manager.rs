use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

const CONFIG_FILES: &[&str] = &[
    "categories.json",
    "projects.json",
    "files.json",
    "preferences.json",
    "favorites.json",
    "recents.json",
];

/// Primary config directory: ~/.config/vori
pub fn config_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("vori"))
        .ok_or_else(|| "Cannot determine config directory".to_string())
}

/// Legacy config directory: ~/.config/code-launcher
fn legacy_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("code-launcher"))
}

/// On first launch, migrate JSON files from code-launcher → vori.
/// Safe to call on every startup: no-ops if vori dir already exists.
pub fn migrate_from_legacy() {
    let Ok(vori_dir) = config_dir() else { return };

    // If vori dir already exists, nothing to do.
    if vori_dir.exists() {
        return;
    }

    let Some(legacy_dir) = legacy_config_dir() else { return };

    // Legacy dir must exist and have at least one JSON file worth migrating.
    if !legacy_dir.exists() {
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&vori_dir) {
        eprintln!("[vori] Failed to create config dir: {e}");
        return;
    }

    for filename in CONFIG_FILES {
        let src = legacy_dir.join(filename);
        if src.exists() {
            let dst = vori_dir.join(filename);
            if let Err(e) = std::fs::copy(&src, &dst) {
                eprintln!("[vori] Failed to migrate {filename}: {e}");
            } else {
                println!("[vori] Migrated {filename} from code-launcher → vori");
            }
        }
    }
}

pub fn load<T: DeserializeOwned + Default>(filename: &str) -> Result<T, String> {
    let path = config_dir()?.join(filename);
    if !path.exists() {
        return Ok(T::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {filename}: {e}"))?;
    if content.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse {filename}: {e}"))
}

pub fn save<T: Serialize>(filename: &str, data: &T) -> Result<(), String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config dir: {e}"))?;
    let path = dir.join(filename);
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write {filename}: {e}"))
}
