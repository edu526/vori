use std::collections::HashMap;
use std::process::Command;

/// (key, binary_command, display_name)
pub const KNOWN_EDITORS: &[(&str, &str, &str)] = &[
    ("vscode",          "code",            "VSCode"),
    ("vscode-insiders", "code-insiders",   "VSCode Insiders"),
    ("cursor",          "cursor",          "Cursor"),
    ("windsurf",        "windsurf",        "Windsurf"),
    ("kiro",            "kiro",            "Kiro"),
    ("zed",             "zed",             "Zed"),
    ("fleet",           "fleet",           "Fleet"),
    ("sublime",         "subl",            "Sublime Text"),
    ("graviton",        "graviton",        "Graviton"),
    ("helix",           "hx",              "Helix"),
    ("neovim",          "nvim",            "Neovim"),
    ("vim",             "vim",             "Vim"),
    ("emacs",           "emacs",           "Emacs"),
    ("kate",            "kate",            "Kate"),
    ("gedit",           "gedit",           "Gedit"),
];

/// Detects installed editors. Returns map of key → binary path.
pub fn detect_editors() -> HashMap<String, String> {
    let mut found = HashMap::new();
    for (key, cmd, _display) in KNOWN_EDITORS {
        if let Ok(output) = Command::new("which").arg(cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    found.insert(key.to_string(), path);
                }
            }
        }
    }
    found
}

/// Returns the display name for a given editor key, or the key itself.
pub fn display_name(key: &str) -> String {
    KNOWN_EDITORS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, _, name)| name.to_string())
        .unwrap_or_else(|| key.to_string())
}
