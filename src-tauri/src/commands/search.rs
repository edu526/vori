use tauri::State;

use crate::services::ranking::{frecency_bonus, text_score};
use crate::state::AppState;

/// Most results returned for a query.
const MAX_RESULTS: usize = 50;
/// Items shown before anything is typed.
const FREQUENT_SHOWN: usize = 8;

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub key: String,
    pub name: String,
    pub result_type: String,
    pub path: Option<String>,
    pub parent: Option<String>, // direct parent category key
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// With a query: matches ranked by relevance plus how often/recently they were opened.
/// With an empty query: the things opened most (frecency), so the search box is useful
/// before typing.
#[tauri::command]
pub fn search(query: String, state: State<AppState>) -> Vec<SearchResult> {
    let q = query.trim().to_string();
    let now = now_secs();
    let usage = state.usage.lock().unwrap().clone();
    let mut scored: Vec<(f64, u8, SearchResult)> = Vec::new();

    // Categories (no path, so no usage): match the last segment first, then the full key.
    if !q.is_empty() {
        let cats = state.categories.lock().unwrap();
        for (key, cat) in cats.iter() {
            let label = key.rsplit('/').next().unwrap_or(key);
            if let Some(score) = text_score(&q, label, Some(key)) {
                scored.push((
                    score,
                    0,
                    SearchResult {
                        key: key.clone(),
                        name: key.clone(),
                        result_type: "category".to_string(),
                        path: None,
                        parent: cat.parent.clone(),
                    },
                ));
            }
        }
    }

    // Projects
    {
        let projs = state.projects.lock().unwrap();
        for (key, proj) in projs.iter() {
            let used = usage.get(&proj.path);
            let score = if q.is_empty() {
                used.map(|_| 0.0)
            } else {
                text_score(&q, key, Some(&proj.path))
            };
            if let Some(score) = score {
                scored.push((
                    score + frecency_bonus(used, now),
                    1,
                    SearchResult {
                        key: key.clone(),
                        name: key.clone(),
                        result_type: "project".to_string(),
                        path: Some(proj.path.clone()),
                        parent: Some(proj.parent.clone()),
                    },
                ));
            }
        }
    }

    // Files
    {
        let files = state.files.lock().unwrap();
        for (key, file) in files.iter() {
            let used = usage.get(&file.path);
            let score = if q.is_empty() {
                used.map(|_| 0.0)
            } else {
                text_score(&q, key, Some(&file.path))
            };
            if let Some(score) = score {
                scored.push((
                    score + frecency_bonus(used, now),
                    2,
                    SearchResult {
                        key: key.clone(),
                        name: key.clone(),
                        result_type: "file".to_string(),
                        path: Some(file.path.clone()),
                        parent: None,
                    },
                ));
            }
        }
    }

    // Best score first; ties keep the old order (categories → projects → files, then A→Z).
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.name.cmp(&b.2.name))
    });
    let limit = if q.is_empty() { FREQUENT_SHOWN } else { MAX_RESULTS };
    scored.into_iter().take(limit).map(|(_, _, r)| r).collect()
}
