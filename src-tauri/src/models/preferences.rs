use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPreferences {
    pub preferred: Option<String>,
    pub available: HashMap<String, String>,
    pub last_detected: Option<String>,
}

impl Default for TerminalPreferences {
    fn default() -> Self {
        Self {
            preferred: None,
            available: HashMap::new(),
            last_detected: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// UI language. `System` follows the operating system.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    System,
    En,
    Es,
}

impl Language {
    /// The concrete language to show: `"en"` or `"es"`.
    pub fn resolve(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::System => {
                let spanish = sys_locale::get_locale()
                    .is_some_and(|l| l.to_lowercase().starts_with("es"));
                if spanish { "es" } else { "en" }
            }
        }
    }
}

fn default_autostart() -> bool {
    true
}

fn default_show_tray() -> bool {
    true
}

fn default_keep_background() -> bool {
    true
}

fn default_hotkey() -> String {
    "Super+Shift+KeyV".to_string()
}

fn default_editor_text_wrap() -> bool {
    false
}

fn default_editor_tab_size() -> u8 {
    2
}

fn default_editor_font_size() -> u8 {
    13
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub default_editor: String,
    pub default_text_editor: Option<String>,
    #[serde(default)]
    pub close_on_open_editor: bool,
    #[serde(default)]
    pub close_on_open_terminal: bool,
    #[serde(default)]
    pub close_on_open_file: bool,
    pub terminal: TerminalPreferences,
    #[serde(default)]
    pub editors_available: HashMap<String, String>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default = "default_show_tray")]
    pub show_tray: bool,
    #[serde(default = "default_keep_background")]
    pub keep_background: bool,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_editor_text_wrap")]
    pub editor_text_wrap: bool,
    #[serde(default = "default_editor_tab_size")]
    pub editor_tab_size: u8,
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: u8,
    /// Named Claude profiles: profile name → `CLAUDE_CONFIG_DIR` path.
    #[serde(default)]
    pub claude_profiles: HashMap<String, String>,
}

fn default_ui_scale() -> f32 {
    1.0
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_editor: "vscode".to_string(),
            default_text_editor: None,
            close_on_open_editor: false,
            close_on_open_terminal: false,
            close_on_open_file: false,
            terminal: TerminalPreferences::default(),
            editors_available: HashMap::new(),
            theme: Theme::default(),
            language: Language::default(),
            autostart: true,
            show_tray: true,
            keep_background: true,
            hotkey: default_hotkey(),
            ui_scale: 1.0,
            editor_text_wrap: false,
            editor_tab_size: 2,
            editor_font_size: 13,
            claude_profiles: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod language_tests {
    use super::*;

    #[test]
    fn explicit_languages_resolve_to_themselves() {
        assert_eq!(Language::En.resolve(), "en");
        assert_eq!(Language::Es.resolve(), "es");
        assert!(matches!(Language::System.resolve(), "en" | "es"));
    }

    #[test]
    fn preferences_without_a_language_default_to_system() {
        // preferences.json written by an older version has no `language` key
        let prefs: Preferences = serde_json::from_str(
            r#"{"default_editor":"vscode","default_text_editor":null,"terminal":{"preferred":null,"available":{},"last_detected":null}}"#,
        )
        .unwrap();
        assert_eq!(prefs.language, Language::System);
        let round: Language = serde_json::from_str("\"es\"").unwrap();
        assert_eq!(round, Language::Es);
    }
}
