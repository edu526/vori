use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Favorites {
    pub projects: Vec<String>,
    pub files: Vec<String>,
    pub categories: Vec<String>,
}
