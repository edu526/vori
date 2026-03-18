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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub default_editor: String,
    pub default_text_editor: Option<String>,
    pub close_on_open: bool,
    pub terminal: TerminalPreferences,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_editor: "vscode".to_string(),
            default_text_editor: None,
            close_on_open: false,
            terminal: TerminalPreferences::default(),
        }
    }
}
