use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::claude_profile::CONFIG_DIR_ENV;

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

/// TOML basic-string literal for `s`.
#[cfg(target_os = "linux")]
fn toml_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// POSIX single-quoted shell word for `s`.
#[cfg(target_os = "linux")]
fn sh_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Warp "Tab Config" that opens one terminal tab in `path` and exports `CLAUDE_CONFIG_DIR`.
#[cfg(target_os = "linux")]
fn warp_tab_config_toml(path: Option<&str>, claude_config_dir: &str) -> String {
    let mut toml = String::from("name = \"Vori launch\"\n\n[[panes]]\nid = \"root\"\ntype = \"terminal\"\n");
    if let Some(p) = path {
        toml.push_str(&format!("directory = {}\n", toml_string(p)));
    }
    let export = format!("export {CONFIG_DIR_ENV}={}", sh_single_quote(claude_config_dir));
    toml.push_str(&format!("commands = [{}]\n", toml_string(&export)));
    toml
}

/// Open a Warp tab through its URI scheme instead of spawning the binary.
///
/// Warp is single-instance: a second `warp-terminal` process hands off to the running one, so
/// neither its environment nor any flag reaches the new shell. A Tab Config opened via
/// `warp://tab_config/<name>` is handled by the running instance and can run startup commands.
#[cfg(target_os = "linux")]
fn open_warp_tab_config(path: Option<&str>, claude_config_dir: &str) -> Result<(), String> {
    let dir = dirs::data_dir()
        .ok_or("no data directory")?
        .join("warp-terminal")
        .join("tab_configs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;

    // Unique per launch so concurrent launches never overwrite each other's config
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let stem = format!("vori_launch_{millis}");
    let file = dir.join(format!("{stem}.toml"));
    std::fs::write(&file, warp_tab_config_toml(path, claude_config_dir))
        .map_err(|e| format!("write {}: {e}", file.display()))?;

    let uri = format!("warp://tab_config/{stem}");
    eprintln!("[vori][terminal] opening Warp tab config: xdg-open {uri:?}");
    let opened = Command::new("xdg-open").arg(&uri).status();
    match opened {
        Ok(status) if status.success() => {
            // Warp has read the file by now; drop it so launches don't pile up in its "+" menu
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(15));
                let _ = std::fs::remove_file(file);
            });
            Ok(())
        }
        other => {
            let _ = std::fs::remove_file(&file);
            Err(format!("xdg-open {uri}: {other:?}"))
        }
    }
}

/// Args that make `stem` start `env CLAUDE_CONFIG_DIR=<dir> $SHELL` instead of its default shell.
///
/// Only for terminals whose "run this command" syntax is known. Setting the variable on the
/// launcher process is not enough for single-instance/server terminals: a new tab is spawned
/// by the already-running process, which never sees our environment. `None` means the
/// terminal has no such flag and we can only rely on environment inheritance.
fn shell_with_config_dir_args(stem: &str, dir: &str) -> Option<Vec<String>> {
    if !cfg!(unix) {
        return None;
    }
    let flag = match stem {
        "gnome-terminal" => Some("--"),
        "konsole" | "alacritty" | "xterm" => Some("-e"),
        "kitty" => None, // takes the command as trailing positional args
        _ => return None,
    };
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut args: Vec<String> = flag.into_iter().map(String::from).collect();
    args.extend(["env".to_string(), format!("{CONFIG_DIR_ENV}={dir}"), shell]);
    Some(args)
}

pub fn open_terminal(
    path: Option<&str>,
    terminal_cmd: &str,
    claude_config_dir: Option<&str>,
) -> Result<(), String> {
    eprintln!("[vori][terminal] open_terminal path={path:?} cmd={terminal_cmd:?} claude_config_dir={claude_config_dir:?}");
    let stem = Path::new(terminal_cmd)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(terminal_cmd);

    #[cfg(target_os = "linux")]
    if let Some(dir) = claude_config_dir {
        if matches!(stem, "warp" | "warp-terminal") {
            match open_warp_tab_config(path, dir) {
                Ok(()) => return Ok(()),
                // Fall through to the plain spawn, which still works when Warp isn't running yet
                Err(e) => eprintln!("[vori][terminal] Warp tab config failed ({e}); falling back to spawn"),
            }
        }
    }

    // On Windows, wt (Windows Terminal) is an App Execution Alias that
    // CreateProcessW cannot launch directly — wrap with `cmd /c`.
    if cfg!(windows) && stem == "wt" {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg("wt");
        if let Some(p) = path {
            cmd.arg("-d").arg(p);
        }
        if let Some(dir) = claude_config_dir {
            cmd.env(CONFIG_DIR_ENV, dir);
        }
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
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
        if let Some(dir) = claude_config_dir {
            cmd.env(CONFIG_DIR_ENV, dir);
            if let Some(args) = shell_with_config_dir_args(stem, dir) {
                cmd.args(args);
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

/// Arguments that make terminal `stem` open in `path` and run `command`, leaving the terminal
/// open afterwards. `command` must already be safe to embed in a shell line (see `scripts`).
/// `None` for terminals whose "run this" syntax isn't known (Warp, tilix, anything on macOS).
fn command_args(stem: &str, path: &str, command: &str) -> Option<Vec<String>> {
    let sh_script = format!("{command}; exec \"${{SHELL:-sh}}\"");
    let owned = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if cfg!(windows) {
        return match stem {
            "powershell" | "pwsh" => Some(owned(&["-NoExit", "-Command", command])),
            "cmd" => Some(owned(&["/k", command])),
            // `wt` is launched through `cmd /c` (see `run_in_terminal`)
            "wt" => Some(owned(&["-d", path, "cmd", "/k", command])),
            _ => None,
        };
    }
    if cfg!(unix) {
        return match stem {
            "gnome-terminal" => Some(owned(&["--working-directory", path, "--", "sh", "-c", &sh_script])),
            "konsole" => Some(owned(&["--workdir", path, "-e", "sh", "-c", &sh_script])),
            "alacritty" | "xterm" => Some(owned(&["-e", "sh", "-c", &sh_script])),
            "kitty" => Some(owned(&["sh", "-c", &sh_script])),
            _ => None,
        };
    }
    None
}

/// Open the terminal in `path` and run `command` there. Returns `Ok(true)` when the command
/// was started, `Ok(false)` when this terminal can't be told to run one: it is then just opened
/// in `path` and the caller should hand the command to the user.
pub fn run_in_terminal(
    path: &str,
    terminal_cmd: &str,
    command: &str,
    claude_config_dir: Option<&str>,
) -> Result<bool, String> {
    eprintln!("[vori][terminal] run_in_terminal path={path:?} cmd={terminal_cmd:?} command={command:?}");
    let stem = Path::new(terminal_cmd)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(terminal_cmd);

    let Some(args) = command_args(stem, path, command) else {
        open_terminal(Some(path), terminal_cmd, claude_config_dir)?;
        return Ok(false);
    };

    let mut cmd = if cfg!(windows) && stem == "wt" {
        let mut c = Command::new("cmd");
        c.arg("/c").arg("wt");
        c
    } else {
        Command::new(terminal_cmd)
    };
    cmd.args(args).current_dir(path);
    if let Some(dir) = claude_config_dir {
        cmd.env(CONFIG_DIR_ENV, dir);
    }
    #[cfg(windows)]
    if stem == "wt" {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: only the `cmd /c` helper, not wt itself
    }

    eprintln!("[vori][terminal] spawning: {:?}", cmd);
    cmd.spawn()
        .map(|_| true)
        .map_err(|e| format!("Failed to launch terminal: {e}"))
}

#[cfg(windows)]
#[cfg(test)]
mod run_tests_windows {
    use super::*;

    #[test]
    fn windows_terminals_get_their_own_run_syntax() {
        let a = |stem| command_args(stem, r"C:\my proj", "pnpm run dev");
        assert_eq!(a("powershell").unwrap(), ["-NoExit", "-Command", "pnpm run dev"]);
        assert_eq!(a("pwsh").unwrap(), ["-NoExit", "-Command", "pnpm run dev"]);
        assert_eq!(a("cmd").unwrap(), ["/k", "pnpm run dev"]);
        assert_eq!(a("wt").unwrap(), ["-d", r"C:\my proj", "cmd", "/k", "pnpm run dev"]);
        assert!(a("warp").is_none());
    }
}

#[cfg(all(test, unix))]
mod run_tests_unix {
    use super::*;

    #[test]
    fn unix_terminals_run_the_command_then_keep_a_shell_open() {
        let script = "pnpm run dev; exec \"${SHELL:-sh}\"";
        let g = command_args("gnome-terminal", "/p", "pnpm run dev").unwrap();
        assert_eq!(g, ["--working-directory", "/p", "--", "sh", "-c", script]);
        let k = command_args("konsole", "/p", "pnpm run dev").unwrap();
        assert_eq!(k, ["--workdir", "/p", "-e", "sh", "-c", script]);
        assert_eq!(command_args("xterm", "/p", "pnpm run dev").unwrap(), ["-e", "sh", "-c", script]);
        assert_eq!(command_args("kitty", "/p", "pnpm run dev").unwrap(), ["sh", "-c", script]);
        assert!(command_args("warp-terminal", "/p", "x").is_none());
        assert!(command_args("tilix", "/p", "x").is_none());
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn args(stem: &str) -> Option<Vec<String>> {
        shell_with_config_dir_args(stem, "/home/u/.claude-work")
    }

    #[test]
    fn exec_syntax_matches_each_terminal() {
        let env = format!("{CONFIG_DIR_ENV}=/home/u/.claude-work");
        assert_eq!(&args("gnome-terminal").unwrap()[..3], ["--", "env", env.as_str()]);
        assert_eq!(&args("xterm").unwrap()[..3], ["-e", "env", env.as_str()]);
        assert_eq!(&args("konsole").unwrap()[..3], ["-e", "env", env.as_str()]);
        assert_eq!(&args("alacritty").unwrap()[..3], ["-e", "env", env.as_str()]);
        // kitty takes the command positionally, with no flag
        assert_eq!(&args("kitty").unwrap()[..2], ["env", env.as_str()]);
    }

    #[test]
    fn terminals_without_a_known_exec_flag_are_left_alone() {
        assert!(args("warp-terminal").is_none());
        assert!(args("tilix").is_none());
    }
}

#[cfg(all(test, target_os = "linux"))]
mod warp_tests {
    use super::*;

    #[test]
    fn tab_config_sets_directory_and_exports_the_profile() {
        let toml = warp_tab_config_toml(Some("/home/u/proj"), "/home/u/.claude-work");
        assert!(toml.contains("directory = \"/home/u/proj\"\n"));
        assert!(toml.contains("commands = [\"export CLAUDE_CONFIG_DIR='/home/u/.claude-work'\"]\n"));
    }

    #[test]
    fn tab_config_without_a_path_has_no_directory() {
        assert!(!warp_tab_config_toml(None, "/x").contains("directory"));
    }

    #[test]
    fn hostile_paths_cannot_break_out_of_the_toml_or_the_shell_word() {
        // A quote in the directory must not end the shell word or the TOML string
        let toml = warp_tab_config_toml(Some("/a\"b\\c"), "/x/it's");
        assert!(toml.contains("directory = \"/a\\\"b\\\\c\"\n"));
        assert!(toml.contains("export CLAUDE_CONFIG_DIR='/x/it'\\\\''s'"));
        assert_eq!(sh_single_quote("it's"), "'it'\\''s'");
        assert_eq!(toml_string("a\nb"), "\"a\\nb\"");
    }
}
