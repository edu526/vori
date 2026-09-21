use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_path: Option<String>,
    /// Name of a Claude profile (key of `Preferences.claude_profiles`). Inherited by children.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub claude_profile: Option<String>,
}

pub type CategoriesMap = HashMap<String, Category>;
