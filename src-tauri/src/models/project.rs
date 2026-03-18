use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub category: String,
    pub subcategory: Option<String>,
}

pub type ProjectsMap = HashMap<String, Project>;
