use std::path::Path;

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

        // Build relative path with forward slashes
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        results.push(ScannedProject {
            name,
            path: path.to_string_lossy().to_string(),
            stack,
            relative_path,
        });
        return; // Don't recurse into detected projects
    }

    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };

    let mut children: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let Ok(meta) = e.metadata() else { return false };
            if !meta.is_dir() {
                return false;
            }
            let fname = e.file_name();
            let name = fname.to_string_lossy().to_string();
            !is_ignored(&name)
        })
        .collect();

    children.sort_by_key(|e| e.file_name());

    for entry in children {
        scan_recursive(&entry.path(), root, depth + 1, max_depth, results);
    }
}

#[tauri::command]
pub fn scan_folder(path: String, max_depth: Option<u32>) -> Vec<ScannedProject> {
    let root = Path::new(&path);
    let depth = max_depth.unwrap_or(MAX_DEPTH);
    let mut results = Vec::new();

    let Ok(entries) = std::fs::read_dir(root) else {
        return results;
    };

    let mut children: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let Ok(meta) = e.metadata() else { return false };
            if !meta.is_dir() {
                return false;
            }
            let fname = e.file_name();
            let name = fname.to_string_lossy().to_string();
            !is_ignored(&name)
        })
        .collect();

    children.sort_by_key(|e| e.file_name());

    for entry in children {
        scan_recursive(&entry.path(), root, 1, depth, &mut results);
    }

    results
}
