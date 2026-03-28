use std::process::Command;

/// Open a path in an editor. `binary` is the full path or command resolved by the detector.
pub fn open_in_editor(path: &str, binary: &str) -> Result<(), String> {
    Command::new(binary)
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch '{binary}': {e}"))?;
    Ok(())
}

pub fn open_workspace_in_editor(paths: &[String], binary: &str) -> Result<(), String> {
    let mut cmd = Command::new(binary);
    for path in paths {
        cmd.arg(path);
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to launch '{binary}': {e}"))?;
    Ok(())
}

pub fn open_file_in_text_editor(path: &str, text_editor: Option<&str>) -> Result<(), String> {
    let editor = text_editor.unwrap_or("xdg-open");
    Command::new(editor)
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch '{editor}': {e}"))?;
    Ok(())
}
