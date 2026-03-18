use std::process::Command;

pub fn open_in_editor(path: &str, editor: &str) -> Result<(), String> {
    let cmd = match editor {
        "kiro" => "kiro",
        _ => "code",
    };
    Command::new(cmd)
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch {cmd}: {e}"))?;
    Ok(())
}

pub fn open_file_in_text_editor(path: &str, text_editor: Option<&str>) -> Result<(), String> {
    let editor = text_editor.unwrap_or("xdg-open");
    Command::new(editor)
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch {editor}: {e}"))?;
    Ok(())
}
