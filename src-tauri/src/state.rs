use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::services::cli::FrontendRequest;
use crate::models::{
    category::CategoriesMap,
    favorites::Favorites,
    file_entry::FilesMap,
    preferences::Preferences,
    project::ProjectsMap,
    recents::RecentsList,
    usage::UsageMap,
};

pub struct AppState {
    pub categories: Mutex<CategoriesMap>,
    pub projects: Mutex<ProjectsMap>,
    pub files: Mutex<FilesMap>,
    pub preferences: Mutex<Preferences>,
    pub favorites: Mutex<Favorites>,
    pub recents: Mutex<RecentsList>,
    /// How often / recently each path was opened; drives search ranking.
    pub usage: Mutex<UsageMap>,
    pub is_autostart: AtomicBool,
    /// Requests from the CLI / deep links waiting for the frontend to pick them up.
    pub pending: Mutex<Vec<FrontendRequest>>,
}

impl AppState {
    pub fn new(
        categories: CategoriesMap,
        projects: ProjectsMap,
        files: FilesMap,
        preferences: Preferences,
        favorites: Favorites,
        recents: RecentsList,
        usage: UsageMap,
        is_autostart: bool,
    ) -> Self {
        Self {
            categories: Mutex::new(categories),
            projects: Mutex::new(projects),
            files: Mutex::new(files),
            preferences: Mutex::new(preferences),
            favorites: Mutex::new(favorites),
            recents: Mutex::new(recents),
            usage: Mutex::new(usage),
            is_autostart: AtomicBool::new(is_autostart),
            pending: Mutex::new(Vec::new()),
        }
    }
}