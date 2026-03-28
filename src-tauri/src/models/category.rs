use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub parent: Option<String>,
}

pub type CategoriesMap = HashMap<String, Category>;
