use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct EditorDef {
    pub key: &'static str,
    pub display: &'static str,
    /// Binary name for PATH lookup (which/where)
    pub bin: &'static str,
    /// Extra paths to check if PATH lookup fails (~ is expanded, %VAR% on Windows)
    pub fallbacks: &'static [&'static str],
}

pub const KNOWN_EDITORS: &[EditorDef] = &[
    EditorDef {
        key: "vscode", display: "VSCode", bin: "code",
        fallbacks: &[
            // Linux
            "~/.local/bin/code",
            "/usr/local/bin/code",
            "/snap/bin/code",
            // macOS
            "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
            "/usr/local/bin/code",
            // Windows
            "%LOCALAPPDATA%/Programs/Microsoft VS Code/bin/code.cmd",
            "%PROGRAMFILES%/Microsoft VS Code/bin/code.cmd",
        ],
    },
    EditorDef {
        key: "vscode-insiders", display: "VSCode Insiders", bin: "code-insiders",
        fallbacks: &[
            "~/.local/bin/code-insiders",
            "/snap/bin/code-insiders",
            "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/bin/code-insiders",
            "%LOCALAPPDATA%/Programs/Microsoft VS Code Insiders/bin/code-insiders.cmd",
        ],
    },
    EditorDef {
        key: "cursor", display: "Cursor", bin: "cursor",
        fallbacks: &[
            // Linux
            "~/.local/bin/cursor",
            "/opt/cursor/cursor",
            "/opt/Cursor/cursor",
            "/snap/bin/cursor",
            "~/.local/share/flatpak/exports/bin/com.todesktop.230313mzl4w4u92",
            // macOS
            "/Applications/Cursor.app/Contents/MacOS/Cursor",
            // Windows
            "%LOCALAPPDATA%/Programs/cursor/Cursor.exe",
            "%APPDATA%/../Local/Programs/cursor/Cursor.exe",
        ],
    },
    EditorDef {
        key: "windsurf", display: "Windsurf", bin: "windsurf",
        fallbacks: &[
            "~/.local/bin/windsurf",
            "/opt/windsurf/windsurf",
            "/snap/bin/windsurf",
            "/Applications/Windsurf.app/Contents/MacOS/Windsurf",
            "%LOCALAPPDATA%/Programs/windsurf/Windsurf.exe",
        ],
    },
    EditorDef {
        key: "kiro", display: "Kiro", bin: "kiro",
        fallbacks: &[
            "~/.local/bin/kiro",
            "/opt/kiro/kiro",
            "/Applications/Kiro.app/Contents/MacOS/Kiro",
            "%LOCALAPPDATA%/Programs/kiro/Kiro.exe",
        ],
    },
    EditorDef {
        key: "zed", display: "Zed", bin: "zed",
        fallbacks: &[
            "~/.local/bin/zed",
            "/usr/local/bin/zed",
            "/Applications/Zed.app/Contents/MacOS/zed",
            "~/.local/share/flatpak/exports/bin/dev.zed.Zed",
        ],
    },
    EditorDef {
        key: "fleet", display: "Fleet", bin: "fleet",
        fallbacks: &[
            "~/.local/bin/fleet",
            "/usr/local/bin/fleet",
            "/Applications/Fleet.app/Contents/MacOS/Fleet",
            "%LOCALAPPDATA%/Programs/Fleet/Fleet.exe",
        ],
    },
    EditorDef {
        key: "sublime", display: "Sublime Text", bin: "subl",
        fallbacks: &[
            "/usr/bin/subl",
            "/opt/sublime_text/subl",
            "/Applications/Sublime Text.app/Contents/SharedSupport/bin/subl",
            "%PROGRAMFILES%/Sublime Text/subl.exe",
            "%PROGRAMFILES%/Sublime Text 3/subl.exe",
        ],
    },
    EditorDef {
        key: "helix", display: "Helix", bin: "hx",
        fallbacks: &[
            "~/.local/bin/hx",
            "/usr/local/bin/hx",
            "/Applications/Helix.app/Contents/MacOS/hx",
        ],
    },
    EditorDef {
        key: "neovim", display: "Neovim", bin: "nvim",
        fallbacks: &[
            "/usr/bin/nvim",
            "/usr/local/bin/nvim",
            "~/.local/bin/nvim",
            "/opt/homebrew/bin/nvim",
        ],
    },
    EditorDef {
        key: "vim", display: "Vim", bin: "vim",
        fallbacks: &[
            "/usr/bin/vim",
            "/usr/local/bin/vim",
            "/opt/homebrew/bin/vim",
        ],
    },
    EditorDef {
        key: "emacs", display: "Emacs", bin: "emacs",
        fallbacks: &[
            "/usr/bin/emacs",
            "/usr/local/bin/emacs",
            "/opt/homebrew/bin/emacs",
            "/Applications/Emacs.app/Contents/MacOS/Emacs",
        ],
    },
    EditorDef {
        key: "kate", display: "Kate", bin: "kate",
        fallbacks: &[
            "/usr/bin/kate",
            "~/.local/share/flatpak/exports/bin/org.kde.kate",
            "/snap/bin/kate",
        ],
    },
    EditorDef {
        key: "gedit", display: "Gedit", bin: "gedit",
        fallbacks: &[
            "/usr/bin/gedit",
            "~/.local/share/flatpak/exports/bin/org.gnome.gedit",
        ],
    },
    EditorDef {
        key: "graviton", display: "Graviton", bin: "graviton",
        fallbacks: &[
            "~/.local/bin/graviton",
            "/opt/graviton/graviton",
            "/Applications/Graviton.app/Contents/MacOS/Graviton",
        ],
    },
    EditorDef {
        key: "antigravity", display: "Antigravity", bin: "antigravity",
        fallbacks: &[
            "/usr/bin/antigravity",
            "~/.local/bin/antigravity",
            "/opt/antigravity/antigravity",
        ],
    },
];

/// Expand ~ and %VAR% in a path string.
fn expand_path(raw: &str) -> Option<PathBuf> {
    let expanded = if raw.starts_with('~') {
        let home = dirs::home_dir()?;
        home.join(&raw[2..])
    } else if raw.contains('%') {
        // Windows-style env vars: %VAR%
        let mut result = raw.to_string();
        for (key, val) in std::env::vars() {
            result = result.replace(&format!("%{key}%"), &val);
        }
        PathBuf::from(result)
    } else {
        PathBuf::from(raw)
    };
    Some(expanded)
}

/// Try to find the editor binary via PATH lookup, then fallback paths.
fn probe(def: &EditorDef) -> Option<String> {
    // 1. Try PATH lookup
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    if let Ok(out) = Command::new(which_cmd).arg(def.bin).output() {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !path.is_empty() {
                println!("[detect] {} → found via PATH: {}", def.key, path);
                return Some(path);
            }
        }
    }

    // 2. Check fallback paths
    for raw in def.fallbacks {
        if let Some(p) = expand_path(raw) {
            if p.is_file() {
                println!("[detect] {} → found via fallback: {}", def.key, p.display());
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }

    println!("[detect] {} → not found", def.key);
    None
}

pub fn detect_editors() -> HashMap<String, String> {
    let mut found = HashMap::new();
    for def in KNOWN_EDITORS {
        if let Some(path) = probe(def) {
            found.insert(def.key.to_string(), path);
        }
    }
    found
}

pub fn display_name(key: &str) -> String {
    KNOWN_EDITORS
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.display.to_string())
        .unwrap_or_else(|| key.to_string())
}
