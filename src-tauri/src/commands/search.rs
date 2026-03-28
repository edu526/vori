use tauri::State;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub key: String,
    pub name: String,
    pub result_type: String,
    pub path: Option<String>,
    pub parent: Option<String>, // direct parent category key
}

#[tauri::command]
pub fn search(query: String, state: State<AppState>) -> Vec<SearchResult> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    // Search categories
    {
        let cats = state.categories.lock().unwrap();
        for (key, cat) in cats.iter() {
            if key.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    key: key.clone(),
                    name: key.clone(),
                    result_type: "category".to_string(),
                    path: None,
                    parent: cat.parent.clone(),
                });
            }
        }
    }

    // Search projects
    {
        let projs = state.projects.lock().unwrap();
        for (key, proj) in projs.iter() {
            if key.to_lowercase().contains(&q) || proj.path.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    key: key.clone(),
                    name: key.clone(),
                    result_type: "project".to_string(),
                    path: Some(proj.path.clone()),
                    parent: Some(proj.parent.clone()),
                });
            }
        }
    }

    // Search files
    {
        let files = state.files.lock().unwrap();
        for (key, file) in files.iter() {
            if key.to_lowercase().contains(&q) || file.path.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    key: key.clone(),
                    name: key.clone(),
                    result_type: "file".to_string(),
                    path: Some(file.path.clone()),
                    parent: None,
                });
            }
        }
    }

    // Sort: categories → projects → files, then alphabetically
    results.sort_by(|a, b| {
        let order = |r: &SearchResult| match r.result_type.as_str() {
            "category" => 0,
            "project" => 1,
            "file" => 2,
            _ => 3,
        };
        order(a).cmp(&order(b)).then_with(|| a.name.cmp(&b.name))
    });

    results
}
