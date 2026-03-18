use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecentType {
    Project,
    File,
    Category,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: RecentType,
    pub timestamp: f64,
}

pub type RecentsList = Vec<RecentItem>;

pub const MAX_RECENTS: usize = 20;
