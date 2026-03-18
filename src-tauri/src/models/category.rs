use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subcategory {
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub icon: String,
    pub subcategories: HashMap<String, Subcategory>,
}

pub type CategoriesMap = HashMap<String, Category>;
