use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// On Windows, .cmd/.bat files cannot be executed directly via CreateProcessW.
/// Wrap them with `cmd /c` so editors like VSCode (code.cmd) launch correctly.
fn build_editor_command(binary: &str) -> Command {
    let mut cmd = if cfg!(windows) {
        let lower = binary.to_ascii_lowercase();
        if lower.ends_with(".cmd") || lower.ends_with(".bat") {
            let mut c = Command::new("cmd");
            c.arg("/c").arg(binary);
            c
        } else {
            Command::new(binary)
        }
    } else {
        Command::new(binary)
    };
    
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    cmd
}

/// Open a path in an editor. `binary` is the full path or command resolved by the detector.
pub fn open_in_editor(path: &str, binary: &str) -> Result<(), String> {
    eprintln!("[vori][editor] open_in_editor path={path:?} binary={binary:?}");
    let mut cmd = build_editor_command(binary);
    cmd.arg(path);
    eprintln!("[vori][editor] spawning: {:?}", cmd);
    match cmd.spawn() {
        Ok(child) => {
            eprintln!("[vori][editor] spawned pid={:?}", child.id());
            Ok(())
        }
        Err(e) => {
            eprintln!("[vori][editor] FAILED to launch '{binary}': {e}");
            Err(format!("Failed to launch '{binary}': {e}"))
        }
    }
}

pub fn open_workspace_in_editor(paths: &[String], binary: &str) -> Result<(), String> {
    eprintln!("[vori][editor] open_workspace_in_editor paths={paths:?} binary={binary:?}");
    let mut cmd = build_editor_command(binary);
    for path in paths {
        cmd.arg(path);
    }
    eprintln!("[vori][editor] spawning: {:?}", cmd);
    cmd.spawn()
        .map_err(|e| format!("Failed to launch '{binary}': {e}"))?;
    Ok(())
}

pub fn open_file_in_text_editor(path: &str, text_editor: Option<&str>) -> Result<(), String> {
    eprintln!("[vori][editor] open_file_in_text_editor path={path:?} editor={text_editor:?}");
    if let Some(editor) = text_editor {
        let mut cmd = build_editor_command(editor);
        cmd.arg(path);
        eprintln!("[vori][editor] spawning: {:?}", cmd);
        cmd.spawn()
            .map_err(|e| format!("Failed to launch '{editor}': {e}"))?;
        return Ok(());
    }

    // No editor configured — use the platform default opener
    #[cfg(windows)]
    {
        eprintln!("[vori][editor] no editor set, using: cmd /c start \"\" {path}");
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "start", "", path]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd.spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    }

    Ok(())
}
