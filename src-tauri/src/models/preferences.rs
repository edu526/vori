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
        }
    }
}
