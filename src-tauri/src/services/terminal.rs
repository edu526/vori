use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

const KNOWN_TERMINALS: &[(&str, &str)] = &[
    // Windows
    ("wt", "wt"),
    ("powershell", "powershell"),
    ("pwsh", "pwsh"),
    ("cmd", "cmd"),
    // Linux
    ("warp", "warp-terminal"),
    ("gnome-terminal", "gnome-terminal"),
    ("konsole", "konsole"),
    ("alacritty", "alacritty"),
    ("kitty", "kitty"),
    ("tilix", "tilix"),
    ("xterm", "xterm"),
];

pub fn detect_terminals() -> HashMap<String, String> {
    let lookup_cmd = if cfg!(windows) { "where" } else { "which" };
    eprintln!("[vori][terminal] detect_terminals using '{lookup_cmd}'");
    let mut found = HashMap::new();
    for (name, cmd) in KNOWN_TERMINALS {
        eprintln!("[vori][terminal] probing '{cmd}' ({name})");
        if let Ok(output) = Command::new(lookup_cmd).arg(cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !path.is_empty() {
                    eprintln!("[vori][terminal] {name} → {path}");
                    found.insert(name.to_string(), path);
                } else {
                    eprintln!("[vori][terminal] {name} → empty path");
                }
            } else {
                eprintln!("[vori][terminal] {name} → not found");
            }
        } else {
            eprintln!("[vori][terminal] {name} → {lookup_cmd} invocation failed");
        }
    }
    eprintln!("[vori][terminal] detected: {:?}", found.keys().collect::<Vec<_>>());
    found
}

pub fn open_terminal(path: Option<&str>, terminal_cmd: &str) -> Result<(), String> {
    eprintln!("[vori][terminal] open_terminal path={path:?} cmd={terminal_cmd:?}");
    let stem = Path::new(terminal_cmd)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(terminal_cmd);

    // On Windows, wt (Windows Terminal) is an App Execution Alias that
    // CreateProcessW cannot launch directly — wrap with `cmd /c`.
    if cfg!(windows) && stem == "wt" {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg("wt");
        if let Some(p) = path {
            cmd.arg("-d").arg(p);
        }
        eprintln!("[vori][terminal] spawning (wt via cmd): {:?}", cmd);
        match cmd.spawn() {
            Ok(_) => { eprintln!("[vori][terminal] wt spawned ok"); Ok(()) }
            Err(e) => {
                eprintln!("[vori][terminal] FAILED: {e}");
                Err(format!("Failed to launch terminal: {e}"))
            }
        }
    } else {
        let mut cmd = Command::new(terminal_cmd);
        if let Some(p) = path {
            match stem {
                "gnome-terminal" => {
                    cmd.arg("--working-directory").arg(p);
                }
                "konsole" => {
                    cmd.arg("--workdir").arg(p);
                }
                "powershell" | "pwsh" => {
                    cmd.arg("-WorkingDirectory").arg(p);
                }
                _ => {
                    cmd.current_dir(p);
                }
            }
        }
        eprintln!("[vori][terminal] spawning: {:?}", cmd);
        match cmd.spawn() {
            Ok(_) => { eprintln!("[vori][terminal] spawned ok"); Ok(()) }
            Err(e) => {
                eprintln!("[vori][terminal] FAILED: {e}");
                Err(format!("Failed to launch terminal: {e}"))
            }
        }
    }
}
