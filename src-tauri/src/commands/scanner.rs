use std::path::{Path, PathBuf};

use crate::services::stack_detector;

const MAX_DEPTH: u32 = 4;

/// Directories to never recurse into
const IGNORED: &[&str] = &[
    // Build / dependency artifacts
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "__pycache__",
    ".cache",
    "vendor",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "coverage",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    "buck-out",
    "bazel-bin",
    // VCS internals
    ".git",
    ".svn",
    ".hg",
    // Python virtual envs
    "venv",
    ".venv",
    "env",
    // Windows system directories
    "System Volume Information",
    "Recovery",
    "PerfLogs",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
];

#[derive(serde::Serialize, Clone)]
pub struct ScannedProject {
    pub name: String,
    pub path: String,
    pub stack: String,
    /// Path relative to the scan root, using '/' as separator.
    /// E.g. "work/backend/api-service"
    pub relative_path: String,
}

#[derive(serde::Serialize, Clone)]
pub struct DetectedWorkspace {
    /// Display name (workspace file stem, e.g. "frontend" for frontend.code-workspace)
    pub name: String,
    /// Folder containing the .code-workspace file
    pub path: String,
    /// Full path to the .code-workspace file
    pub workspace_file: String,
    /// Folder path relative to scan root, '/' separated
    pub relative_path: String,
}

fn is_ignored(name: &str) -> bool {
    if IGNORED.contains(&name) {
        return true;
    }
    // Skip hidden dirs (dot-prefix) except VCS roots
    if name.starts_with('.') && name != ".git" {
        return true;
    }
    // Skip Windows system/recycle dirs ($RECYCLE.BIN, $SysReset, etc.)
    if name.starts_with('$') {
        return true;
    }
    false
}

/// Sorted list of non-ignored subdirectory paths. Returns empty on read errors.
fn list_subdirs(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut children: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| {
            let Ok(meta) = e.metadata() else { return false };
            if !meta.is_dir() {
                return false;
            }
            !is_ignored(&e.file_name().to_string_lossy())
        })
        .map(|e| e.path())
        .collect();
    children.sort();
    children
}

/// Build a forward-slash relative path from `root` to `path`.
fn relative_path_str(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn scan_recursive(
    path: &Path,
    root: &Path,
    depth: u32,
    max_depth: u32,
    results: &mut Vec<ScannedProject>,
) {
    if depth > max_depth {
        return;
    }

    let stack = stack_detector::detect_stack(path);
    if stack != "unknown" {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        results.push(ScannedProject {
            name,
            path: path.to_string_lossy().to_string(),
            stack,
            relative_path: relative_path_str(path, root),
        });
        return; // Don't recurse into detected projects
    }

    for child in list_subdirs(path) {
        scan_recursive(&child, root, depth + 1, max_depth, results);
    }
}

fn find_workspace_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let Ok(meta) = e.metadata() else { return None };
            if !meta.is_file() {
                return None;
            }
            let p = e.path();
            match p.extension().and_then(|s| s.to_str()) {
                Some(ext) if ext == "code-workspace" => Some(p),
                _ => None,
            }
        })
        .collect();
    found.sort();
    found
}

fn scan_workspaces_recursive(
    path: &Path,
    root: &Path,
    depth: u32,
    max_depth: u32,
    results: &mut Vec<DetectedWorkspace>,
) {
    if depth > max_depth {
        return;
    }

    let workspace_files = find_workspace_files(path);
    if !workspace_files.is_empty() {
        let rel = relative_path_str(path, root);
        for ws in workspace_files {
            let name = ws
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("workspace")
                .to_string();
            results.push(DetectedWorkspace {
                name,
                path: path.to_string_lossy().to_string(),
                workspace_file: ws.to_string_lossy().to_string(),
                relative_path: rel.clone(),
            });
        }
        return;
    }

    for child in list_subdirs(path) {
        scan_workspaces_recursive(&child, root, depth + 1, max_depth, results);
    }
}

fn scan_folder_blocking(path: &str, max_depth: Option<u32>) -> Vec<ScannedProject> {
    let root = Path::new(path);
    let depth = max_depth.unwrap_or(MAX_DEPTH);
    let mut results = Vec::new();
    for child in list_subdirs(root) {
        scan_recursive(&child, root, 1, depth, &mut results);
    }
    results
}

/// Async so a scan of a big tree runs on a worker thread instead of freezing the window
/// (this is also polled in the background to spot new folders).
#[tauri::command]
pub async fn scan_folder(path: String, max_depth: Option<u32>) -> Vec<ScannedProject> {
    tauri::async_runtime::spawn_blocking(move || scan_folder_blocking(&path, max_depth))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn detect_workspaces_in_folder(path: String, max_depth: Option<u32>) -> Vec<DetectedWorkspace> {
    let root = Path::new(&path);
    let depth = max_depth.unwrap_or(MAX_DEPTH);
    let mut results = Vec::new();
    for child in list_subdirs(root) {
        scan_workspaces_recursive(&child, root, 1, depth, &mut results);
    }
    results
}