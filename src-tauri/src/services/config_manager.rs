use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{de::DeserializeOwned, Serialize};

/// Files that were unreadable at startup, reported to the UI so the user knows why
/// something looks empty (the originals are kept next to them as `*.corrupt-*.bak`).
static RECOVERY_NOTES: Mutex<Vec<RecoveryNote>> = Mutex::new(Vec::new());

/// A config file that could not be parsed at startup and was reset.
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryNote {
    pub file: String,
    /// Name the unreadable original was moved to, if that worked.
    pub backup: Option<String>,
}

pub fn recovery_notes() -> Vec<RecoveryNote> {
    RECOVERY_NOTES.lock().unwrap().clone()
}

const CONFIG_FILES: &[&str] = &[
    "categories.json",
    "projects.json",
    "files.json",
    "preferences.json",
    "favorites.json",
    "recents.json",
];

/// Environment variable that moves the config directory, e.g. into a synced folder.
pub const CONFIG_DIR_ENV: &str = "VORI_CONFIG_DIR";

fn resolve_config_dir(custom: Option<std::ffi::OsString>, base: Option<PathBuf>) -> Option<PathBuf> {
    match custom.filter(|v| !v.is_empty()) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => base.map(|d| d.join("vori")),
    }
}

/// Primary config directory: `$VORI_CONFIG_DIR` if set, otherwise ~/.config/vori
pub fn config_dir() -> Result<PathBuf, String> {
    resolve_config_dir(std::env::var_os(CONFIG_DIR_ENV), dirs::config_dir())
        .ok_or_else(|| "Cannot determine config directory".to_string())
}

/// Legacy config directory: ~/.config/code-launcher
fn legacy_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("code-launcher"))
}

/// On first launch, migrate JSON files from code-launcher → vori.
/// Safe to call on every startup: no-ops if vori dir already exists.
pub fn migrate_from_legacy() {
    // A custom config dir is chosen on purpose: never fill it from an old install.
    if std::env::var_os(CONFIG_DIR_ENV).is_some_and(|v| !v.is_empty()) {
        return;
    }
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
                    let _ = write_atomic(&projs_path, s.as_bytes());
                }
            }
        }
    }

    if let Ok(s) = serde_json::to_string_pretty(&serde_json::Value::Object(new_cats)) {
        let _ = write_atomic(&cats_path, s.as_bytes());
        println!("[vori] Migration complete.");
    }
}

/// Write `content` to `path` without ever leaving a half-written file behind:
/// write and flush a sibling temp file, then rename it over the target.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), String> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(content)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result.map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

fn load_from<T: DeserializeOwned + Default>(dir: &Path, filename: &str) -> Result<T, String> {
    let path = dir.join(filename);
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

fn save_to<T: Serialize>(dir: &Path, filename: &str, data: &T) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {e}"))?;
    write_atomic(&dir.join(filename), content.as_bytes())
}

/// Like `load_from`, but a file that cannot be parsed is moved aside (never deleted) and the
/// default is returned. Falling back to the default without this would make the next
/// save silently overwrite the user's data.
fn load_or_recover_from<T: DeserializeOwned + Default>(dir: &Path, filename: &str) -> T {
    match load_from(dir, filename) {
        Ok(v) => v,
        Err(e) => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = format!("{filename}.corrupt-{ts}.bak");
            let kept = std::fs::rename(dir.join(filename), dir.join(&backup)).is_ok();
            eprintln!("[vori] {e}. Reset {filename}{}", if kept { format!(", original kept as {backup}") } else { String::new() });
            RECOVERY_NOTES.lock().unwrap().push(RecoveryNote {
                file: filename.to_string(),
                backup: kept.then_some(backup),
            });
            T::default()
        }
    }
}

pub fn load<T: DeserializeOwned + Default>(filename: &str) -> Result<T, String> {
    load_from(&config_dir()?, filename)
}

pub fn save<T: Serialize>(filename: &str, data: &T) -> Result<(), String> {
    save_to(&config_dir()?, filename, data)
}

/// Startup loader: never fails, never loses a corrupt file.
pub fn load_or_recover<T: DeserializeOwned + Default>(filename: &str) -> T {
    match config_dir() {
        Ok(dir) => load_or_recover_from(&dir, filename),
        Err(_) => T::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vori-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn custom_config_dir_wins_and_blank_is_ignored() {
        let base = Some(PathBuf::from("/home/u/.config"));
        assert_eq!(
            resolve_config_dir(Some("/sync/vori".into()), base.clone()),
            Some(PathBuf::from("/sync/vori"))
        );
        assert_eq!(resolve_config_dir(None, base.clone()), Some(PathBuf::from("/home/u/.config/vori")));
        assert_eq!(resolve_config_dir(Some("".into()), base), Some(PathBuf::from("/home/u/.config/vori")));
        assert_eq!(resolve_config_dir(None, None), None);
    }

    #[test]
    fn save_replaces_existing_file_and_leaves_no_temp() {
        let dir = temp_dir("atomic");
        let mut a = HashMap::new();
        a.insert("one".to_string(), 1);
        save_to(&dir, "x.json", &a).unwrap();
        a.insert("two".to_string(), 2);
        save_to(&dir, "x.json", &a).unwrap();

        let back: HashMap<String, i32> = load_from(&dir, "x.json").unwrap();
        assert_eq!(back.len(), 2);
        assert!(!dir.join("x.json.tmp").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_file_is_moved_aside_not_overwritten() {
        let dir = temp_dir("corrupt");
        std::fs::write(dir.join("projects.json"), "{ \"a\": { truncated").unwrap();

        let loaded: HashMap<String, i32> = load_or_recover_from(&dir, "projects.json");
        assert!(loaded.is_empty());
        assert!(!dir.join("projects.json").exists());
        let backups: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("projects.json.corrupt-"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            std::fs::read_to_string(backups[0].path()).unwrap(),
            "{ \"a\": { truncated"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_or_empty_file_is_just_the_default() {
        let dir = temp_dir("missing");
        let a: HashMap<String, i32> = load_or_recover_from(&dir, "nope.json");
        assert!(a.is_empty());
        std::fs::write(dir.join("empty.json"), "  \n").unwrap();
        let b: HashMap<String, i32> = load_or_recover_from(&dir, "empty.json");
        assert!(b.is_empty());
        assert!(dir
            .read_dir()
            .unwrap()
            .all(|e| !e.unwrap().file_name().to_string_lossy().contains("corrupt")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
