//! Export / import of Vori's data as a single JSON file.
//!
//! Preferences are only partly portable: editor and terminal paths, the hotkey and
//! background behaviour belong to one machine, so an import applies just the harmless
//! subset (see `merge_portable_preferences`).

use serde::{Deserialize, Serialize};

use crate::models::{
    category::CategoriesMap, favorites::Favorites, file_entry::FilesMap,
    preferences::Preferences, project::ProjectsMap, recents::RecentsList,
};

pub const FORMAT: &str = "vori-backup";
pub const FORMAT_VERSION: u32 = 1;

/// Larger than any real config; guards against importing the wrong file by mistake.
pub const MAX_BACKUP_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    pub format: String,
    pub version: u32,
    /// Unix seconds.
    pub exported_at: u64,
    #[serde(default)]
    pub categories: CategoriesMap,
    #[serde(default)]
    pub projects: ProjectsMap,
    #[serde(default)]
    pub files: FilesMap,
    #[serde(default)]
    pub favorites: Favorites,
    #[serde(default)]
    pub recents: RecentsList,
    #[serde(default)]
    pub preferences: Option<Preferences>,
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Backup {
    pub fn new(
        categories: CategoriesMap,
        projects: ProjectsMap,
        files: FilesMap,
        favorites: Favorites,
        recents: RecentsList,
        preferences: Preferences,
    ) -> Self {
        Self {
            format: FORMAT.to_string(),
            version: FORMAT_VERSION,
            exported_at: now_secs(),
            categories,
            projects,
            files,
            favorites,
            recents,
            preferences: Some(preferences),
        }
    }
}

/// Parse and validate a backup file's text.
pub fn parse(text: &str) -> Result<Backup, String> {
    let backup: Backup = serde_json::from_str(text).map_err(|e| {
        format!("This file isn't a valid Vori backup ({e})")
    })?;
    if backup.format != FORMAT {
        return Err("This file isn't a Vori backup".to_string());
    }
    if backup.version > FORMAT_VERSION {
        return Err("This backup was made by a newer version of Vori. Update Vori and try again.".to_string());
    }
    Ok(backup)
}

/// Apply the machine-independent parts of `imported` onto `current`.
pub fn merge_portable_preferences(current: &mut Preferences, imported: &Preferences) {
    current.theme = imported.theme.clone();
    current.language = imported.language.clone();
    current.ui_scale = imported.ui_scale;
    current.editor_text_wrap = imported.editor_text_wrap;
    current.editor_tab_size = imported.editor_tab_size;
    current.editor_font_size = imported.editor_font_size;
    current.close_on_open_editor = imported.close_on_open_editor;
    current.close_on_open_terminal = imported.close_on_open_terminal;
    current.close_on_open_file = imported.close_on_open_file;
    // Editors are per machine: keep the choice only if this machine has that editor.
    if current.editors_available.contains_key(&imported.default_editor) {
        current.default_editor = imported.default_editor.clone();
    }
    if let Some(text_editor) = &imported.default_text_editor {
        if current.editors_available.contains_key(text_editor) {
            current.default_text_editor = Some(text_editor.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{category::Category, preferences::Theme, project::Project};

    fn sample() -> Backup {
        let mut categories = CategoriesMap::new();
        categories.insert("work".into(), Category { parent: None, source_path: Some("/w".into()), claude_profile: None });
        let mut projects = ProjectsMap::new();
        projects.insert(
            "api".into(),
            Project { path: "/w/api".into(), parent: "work".into(), stack: Some("rust".into()), claude_profile: None },
        );
        Backup::new(categories, projects, FilesMap::new(), Favorites::default(), Vec::new(), Preferences::default())
    }

    #[test]
    fn a_backup_survives_a_round_trip() {
        let json = serde_json::to_string_pretty(&sample()).unwrap();
        let back = parse(&json).unwrap();
        assert_eq!(back.categories["work"].source_path.as_deref(), Some("/w"));
        assert_eq!(back.projects["api"].stack.as_deref(), Some("rust"));
        assert!(back.preferences.is_some());
    }

    #[test]
    fn foreign_and_future_files_are_rejected_with_a_clear_message() {
        assert!(parse("not json").unwrap_err().contains("valid Vori backup"));
        assert!(parse("{\"projects\": {}}").is_err()); // a bare projects.json

        let mut other = sample();
        other.format = "something-else".into();
        let json = serde_json::to_string(&other).unwrap();
        assert!(parse(&json).unwrap_err().contains("isn't a Vori backup"));

        let mut future = sample();
        future.version = FORMAT_VERSION + 1;
        let json = serde_json::to_string(&future).unwrap();
        assert!(parse(&json).unwrap_err().contains("newer version"));
    }

    #[test]
    fn sections_missing_from_a_backup_import_as_empty() {
        let json = format!("{{\"format\":\"{FORMAT}\",\"version\":1,\"exported_at\":0}}");
        let back = parse(&json).unwrap();
        assert!(back.projects.is_empty() && back.categories.is_empty() && back.preferences.is_none());
    }

    #[test]
    fn only_portable_preferences_are_applied() {
        let mut current = Preferences::default();
        current.editors_available.insert("cursor".into(), "C:/cursor.exe".into());
        current.hotkey = "Ctrl+Alt+KeyV".into();
        current.autostart = false;
        current.terminal.preferred = Some("wt".into());

        let mut imported = Preferences::default();
        imported.theme = Theme::Dark;
        imported.ui_scale = 1.3;
        imported.editor_font_size = 16;
        imported.default_editor = "cursor".into();
        imported.hotkey = "Super+KeyX".into();
        imported.autostart = true;
        imported.terminal.preferred = Some("kitty".into());
        imported.editors_available.insert("kitty-editor".into(), "/usr/bin/x".into());

        merge_portable_preferences(&mut current, &imported);

        assert_eq!(current.theme, Theme::Dark);
        assert_eq!(current.ui_scale, 1.3);
        assert_eq!(current.editor_font_size, 16);
        assert_eq!(current.default_editor, "cursor"); // this machine has it
        // machine-specific settings stay as they were
        assert_eq!(current.hotkey, "Ctrl+Alt+KeyV");
        assert!(!current.autostart);
        assert_eq!(current.terminal.preferred.as_deref(), Some("wt"));
        assert!(!current.editors_available.contains_key("kitty-editor"));
    }

    #[test]
    fn default_editor_is_kept_when_this_machine_lacks_the_imported_one() {
        let mut current = Preferences::default(); // vscode, none detected
        let mut imported = Preferences::default();
        imported.default_editor = "zed".into();
        merge_portable_preferences(&mut current, &imported);
        assert_eq!(current.default_editor, "vscode");
    }
}
