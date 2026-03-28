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

/// Migrate categories.json and projects.json from the old 2-level format to the new N-level flat format.
/// Detects old format by the presence of "subcategories" inside a category entry.
/// Safe to call on every startup — no-ops if already in new format.
pub fn migrate_to_flat_format() {
    let Ok(dir) = config_dir() else { return };

    let cats_path = dir.join("categories.json");
    let projs_path = dir.join("projects.json");

    // Read categories
    let cats_content = match std::fs::read_to_string(&cats_path) {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return,
    };
    let cats_val: serde_json::Value = match serde_json::from_str(&cats_content) {
        Ok(v) => v,
        _ => return,
    };
    let Some(cats_obj) = cats_val.as_object() else { return };

    // Detect old format: any category has "subcategories" key
    let needs_migration = cats_obj.values().any(|v| v.get("subcategories").is_some());
    if !needs_migration { return; }

    println!("[vori] Migrating categories/projects to N-level flat format...");

    // Build new categories map
    let mut new_cats = serde_json::Map::new();
    for (cat_key, cat_val) in cats_obj {
        new_cats.insert(cat_key.clone(), serde_json::json!({ "parent": null }));
        if let Some(subs) = cat_val.get("subcategories").and_then(|s| s.as_object()) {
            for sub_key in subs.keys() {
                new_cats.insert(sub_key.clone(), serde_json::json!({ "parent": cat_key }));
            }
        }
    }

    // Migrate projects
    if let Ok(projs_content) = std::fs::read_to_string(&projs_path) {
        if let Ok(projs_val) = serde_json::from_str::<serde_json::Value>(&projs_content) {
            if let Some(projs_obj) = projs_val.as_object() {
                let mut new_projs = serde_json::Map::new();
                for (proj_key, proj) in projs_obj {
                    // Old format has "category" field; new format has "parent"
                    if proj.get("parent").is_some() {
                        new_projs.insert(proj_key.clone(), proj.clone());
                    } else {
                        let parent = proj.get("subcategory")
                            .and_then(|s| s.as_str())
                            .filter(|s| !s.is_empty())
                            .or_else(|| proj.get("category").and_then(|c| c.as_str()))
                            .unwrap_or("")
                            .to_string();
                        let path = proj.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string();
                        new_projs.insert(proj_key.clone(), serde_json::json!({ "path": path, "parent": parent }));
                    }
                }
                if let Ok(s) = serde_json::to_string_pretty(&new_projs) {
                    let _ = std::fs::write(&projs_path, s);
                }
            }
        }
    }

    if let Ok(s) = serde_json::to_string_pretty(&serde_json::Value::Object(new_cats)) {
        let _ = std::fs::write(&cats_path, s);
        println!("[vori] Migration complete.");
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
