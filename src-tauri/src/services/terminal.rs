use std::collections::HashMap;
use std::process::Command;

const KNOWN_TERMINALS: &[(&str, &str)] = &[
    ("warp", "warp-terminal"),
    ("gnome-terminal", "gnome-terminal"),
    ("konsole", "konsole"),
    ("alacritty", "alacritty"),
    ("kitty", "kitty"),
    ("tilix", "tilix"),
    ("xterm", "xterm"),
];

pub fn detect_terminals() -> HashMap<String, String> {
    let mut found = HashMap::new();
    for (name, cmd) in KNOWN_TERMINALS {
        if let Ok(output) = Command::new("which").arg(cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    found.insert(name.to_string(), path);
                }
            }
        }
    }
    found
}

pub fn open_terminal(path: Option<&str>, terminal_cmd: &str) -> Result<(), String> {
    let mut cmd = Command::new(terminal_cmd);
    if let Some(p) = path {
        match terminal_cmd {
            "gnome-terminal" => {
                cmd.arg("--working-directory").arg(p);
            }
            "konsole" => {
                cmd.arg("--workdir").arg(p);
            }
            _ => {
                cmd.current_dir(p);
            }
        }
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to launch terminal: {e}"))?;
    Ok(())
}
