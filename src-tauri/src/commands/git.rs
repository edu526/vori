use std::collections::HashMap;

use crate::services::git::{self, GitInfo};

/// How many `git status` processes run at once.
const MAX_PARALLEL: usize = 6;

/// Git state for each path that is a repository. Paths that aren't (or that git can't read
/// in time) are simply absent from the result.
#[tauri::command]
pub async fn get_git_info(paths: Vec<String>) -> Result<HashMap<String, GitInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = HashMap::new();
        for chunk in paths.chunks(MAX_PARALLEL) {
            let found: Vec<(String, GitInfo)> = std::thread::scope(|scope| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|path| scope.spawn(move || git::read_info(path).map(|i| (path.clone(), i))))
                    .collect();
                handles.into_iter().filter_map(|h| h.join().ok().flatten()).collect()
            });
            result.extend(found);
        }
        result
    })
    .await
    .map_err(|e| e.to_string())
}

/// Clone a repository into `<dest_parent>/<repo name>`; returns the new folder.
#[tauri::command]
pub async fn git_clone(url: String, dest_parent: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git::clone(&url, &dest_parent))
        .await
        .map_err(|e| e.to_string())?
}
