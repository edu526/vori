use tauri::State;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub key: String,
    pub name: String,
    pub result_type: String,
    pub path: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
}

#[tauri::command]
pub fn search(query: String, state: State<AppState>) -> Vec<SearchResult> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    // Search categories & subcategories
    {
        let cats = state.categories.lock().unwrap();
        for (key, cat) in cats.iter() {
            if key.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    key: key.clone(),
                    name: key.clone(),
                    result_type: "category".to_string(),
                    path: None,
                    category: None,
                    subcategory: None,
                });
            }
            for (sub_key, _) in cat.subcategories.iter() {
                if sub_key.to_lowercase().contains(&q) {
                    results.push(SearchResult {
                        key: format!("{key}/{sub_key}"),
                        name: sub_key.clone(),
                        result_type: "subcategory".to_string(),
                        path: None,
                        category: Some(key.clone()),
                        subcategory: Some(sub_key.clone()),
                    });
                }
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
                    category: Some(proj.category.clone()),
                    subcategory: proj.subcategory.clone(),
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
                    category: None,
                    subcategory: None,
                });
            }
        }
    }

    // Sort: categories → subcategories → projects → files, then alphabetically
    results.sort_by(|a, b| {
        let order = |r: &SearchResult| match r.result_type.as_str() {
            "category" => 0,
            "subcategory" => 1,
            "project" => 2,
            "file" => 3,
            _ => 4,
        };
        order(a).cmp(&order(b)).then_with(|| a.name.cmp(&b.name))
    });

    results
}
