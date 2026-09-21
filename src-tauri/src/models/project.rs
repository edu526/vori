use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub parent: String, // key of parent category node (any depth)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stack: Option<String>,
    /// Overrides the profile inherited from the parent category chain.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub claude_profile: Option<String>,
}

pub type ProjectsMap = HashMap<String, Project>;
