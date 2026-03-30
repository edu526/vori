use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub exec: String,
}

pub fn get_installed_apps() -> Vec<InstalledApp> {
    #[cfg(target_os = "linux")]
    return get_apps_linux();

    #[cfg(target_os = "macos")]
    return get_apps_macos();

    #[cfg(target_os = "windows")]
    return get_apps_windows();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return vec![];
}

// ── Linux ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn get_apps_linux() -> Vec<InstalledApp> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        for d in xdg.split(':') {
            dirs.push(PathBuf::from(d).join("applications"));
        }
    }

    let mut apps: Vec<InstalledApp> = dirs
        .iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("desktop"))
        .filter_map(|e| parse_desktop_file(&e.path()))
        .collect();

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

#[cfg(target_os = "linux")]
fn parse_desktop_file(path: &std::path::Path) -> Option<InstalledApp> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut is_application = false;
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line[5..].trim().to_string());
        } else if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(clean_exec(&line[5..]));
        } else if line == "Type=Application" {
            is_application = true;
        } else if line == "NoDisplay=true" || line == "Hidden=true" {
            no_display = true;
        }
    }

    if !is_application || no_display {
        return None;
    }
    let exec = exec?;
    if exec.is_empty() {
        return None;
    }
    Some(InstalledApp { name: name?, exec })
}

#[cfg(target_os = "linux")]
fn clean_exec(exec: &str) -> String {
    // Remove field codes (%f %u %F %U %i %c %k %d %D %n %N %v %m) and extra flags
    exec.split_whitespace()
        .filter(|p| !p.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

// ── macOS ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn get_apps_macos() -> Vec<InstalledApp> {
    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Applications"));
    }

    let mut apps: Vec<InstalledApp> = dirs
        .iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("app"))
        .filter_map(|e| app_from_bundle(&e.path()))
        .collect();

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[cfg(target_os = "macos")]
fn app_from_bundle(bundle: &std::path::Path) -> Option<InstalledApp> {
    let name = bundle.file_stem()?.to_string_lossy().to_string();
    let macos_dir = bundle.join("Contents/MacOS");
    // The main binary often matches the bundle name
    let primary = macos_dir.join(&name);
    if primary.is_file() {
        return Some(InstalledApp { name, exec: primary.to_string_lossy().to_string() });
    }
    // Fallback: first file in Contents/MacOS
    let bin = std::fs::read_dir(&macos_dir).ok()?.flatten()
        .find(|e| e.path().is_file())?;
    Some(InstalledApp { name, exec: bin.path().to_string_lossy().to_string() })
}

// ── Windows ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn get_apps_windows() -> Vec<InstalledApp> {
    let mut dirs: Vec<PathBuf> = vec![];
    for var in &["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
        if let Ok(val) = std::env::var(var) {
            dirs.push(PathBuf::from(val.clone()));
            if *var == "LOCALAPPDATA" {
                dirs.push(PathBuf::from(val).join("Programs"));
            }
        }
    }

    let mut apps = vec![];
    for dir in &dirs {
        scan_exes(dir, &mut apps, 3);
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

#[cfg(target_os = "windows")]
fn scan_exes(dir: &std::path::Path, apps: &mut Vec<InstalledApp>, depth: u8) {
    if depth == 0 { return; }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_exes(&path, apps, depth - 1);
        } else if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            if let Some(stem) = path.file_stem() {
                apps.push(InstalledApp {
                    name: stem.to_string_lossy().to_string(),
                    exec: path.to_string_lossy().to_string(),
                });
            }
        }
    }
}
