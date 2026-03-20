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
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default = "default_show_tray")]
    pub show_tray: bool,
    #[serde(default = "default_keep_background")]
    pub keep_background: bool,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
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
            autostart: true,
            show_tray: true,
            keep_background: true,
            hotkey: default_hotkey(),
        }
    }
}
