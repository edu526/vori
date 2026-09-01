use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const MAX_BYTES: u64 = 5 * 1024 * 1024;

fn io_err(e: io::Error) -> String {
    format!("io error: {e}")
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    let meta = fs::metadata(&path).map_err(io_err)?;
    if meta.len() > MAX_BYTES {
        return Err(format!(
            "file too large: {} bytes (max {})",
            meta.len(),
            MAX_BYTES
        ));
    }
    fs::read_to_string(&path).map_err(io_err)
}

#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(io_err)
}

#[tauri::command]
pub fn write_text_file_elevated(path: String, content: String) -> Result<(), String> {
    if fs::write(&path, &content).is_ok() {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp = std::env::temp_dir().join(format!("vori-{pid}-{ts}.tmp"));
    fs::write(&temp, &content).map_err(|e| format!("temp write: {e}"))?;

    let result = elevate_and_write(&exe, &path, &temp);
    let _ = fs::remove_file(&temp);
    result
}

#[cfg(windows)]
fn elevate_and_write(exe: &PathBuf, target: &str, temp: &PathBuf) -> Result<(), String> {
    let ps = format!(
        "$p = Start-Process -Verb RunAs -Wait -PassThru -FilePath '{}' -ArgumentList @('--vori-write','{}','{}'); exit $p.ExitCode",
        ps_quote(&exe.display().to_string()),
        ps_quote(target),
        ps_quote(&temp.display().to_string()),
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .status()
        .map_err(|e| format!("spawn powershell: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("elevated write exited with {status}"))
    }
}

#[cfg(windows)]
fn ps_quote(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(target_os = "macos")]
fn elevate_and_write(exe: &PathBuf, target: &str, temp: &PathBuf) -> Result<(), String> {
    let inner = format!(
        "{} --vori-write {} {}",
        sh_quote(exe.to_str().unwrap_or_default()),
        sh_quote(target),
        sh_quote(temp.to_str().unwrap_or_default())
    );
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        inner.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let status = Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| format!("spawn osascript: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("osascript exited with {status}"))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(target_os = "linux")]
fn elevate_and_write(exe: &PathBuf, target: &str, temp: &PathBuf) -> Result<(), String> {
    let inner = format!(
        "{} --vori-write {} {}",
        sh_quote(exe.to_str().unwrap_or_default()),
        sh_quote(target),
        sh_quote(temp.to_str().unwrap_or_default())
    );
    if let Ok(s) = Command::new("pkexec").args(["sh", "-c", &inner]).status() {
        if s.success() {
            return Ok(());
        }
    }
    if let Ok(s) = Command::new("sudo").args(["-n", "sh", "-c", &inner]).status() {
        if s.success() {
            return Ok(());
        }
    }
    Err("elevation failed — install pkexec or configure passwordless sudo, or run Vori as root".to_string())
}