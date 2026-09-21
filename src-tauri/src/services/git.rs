use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Reading status must never stall the UI (network drives, huge repos).
const STATUS_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct GitInfo {
    /// Branch name, or `detached@<short sha>` on a detached HEAD.
    pub branch: String,
    /// Tracked files with uncommitted changes (untracked files are not scanned: too slow).
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
}

fn git() -> Command {
    let mut cmd = Command::new("git");
    cmd.env("GIT_OPTIONAL_LOCKS", "0")
        // Never wait for a credentials / host-key prompt: there is no terminal to answer it.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .stdin(Stdio::null());
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// Parse `git status --porcelain=v2 --branch` output. `None` when it has no branch header.
pub fn parse_status(output: &str) -> Option<GitInfo> {
    let mut head: Option<String> = None;
    let mut oid = String::new();
    let mut info = GitInfo::default();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            head = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.oid ") {
            oid = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for part in rest.split_whitespace() {
                if let Some(n) = part.strip_prefix('+') {
                    info.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    info.behind = n.parse().unwrap_or(0);
                }
            }
        } else if !line.is_empty() && !line.starts_with('#') {
            info.dirty = true;
        }
    }

    let head = head?;
    info.branch = if head == "(detached)" {
        let short: String = oid.chars().take(7).collect();
        format!("detached@{short}")
    } else {
        head
    };
    Some(info)
}

/// Run a command, killing it after `timeout`. Returns stdout only on success.
fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Option<Vec<u8>> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    // Drain stdout on its own thread so a large status can't fill the pipe and stall git.
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = reader.join().ok()?;
                return status.success().then_some(out);
            }
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(15)),
            Err(_) => return None,
        }
    }
}

/// Branch and state of the repository at `path`; `None` if it isn't a git repo (or git
/// is missing / too slow).
pub fn read_info(path: &str) -> Option<GitInfo> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return None;
    }
    let mut cmd = git();
    cmd.arg("-C")
        .arg(dir)
        .args(["--no-optional-locks", "status", "--porcelain=v2", "--branch", "--untracked-files=no"]);
    let out = run_with_timeout(cmd, STATUS_TIMEOUT)?;
    parse_status(&String::from_utf8_lossy(&out))
}

/// Folder name git would use for `url`: `https://host/u/repo.git` and `git@host:u/repo` → `repo`.
pub fn repo_name_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let last = trimmed.rsplit(|c| c == '/' || c == ':').next()?;
    let name = last.strip_suffix(".git").unwrap_or(last);
    if name.is_empty() || name == "." || name == ".." || name.contains('\\') {
        return None;
    }
    Some(name.to_string())
}

/// Clone `url` into `<dest_parent>/<repo name>` and return the new folder.
pub fn clone(url: &str, dest_parent: &str) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Repository URL is required".to_string());
    }
    // A leading dash would be read by git as an option.
    if url.starts_with('-') || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("That doesn't look like a valid repository URL".to_string());
    }
    let name = repo_name_from_url(url)
        .ok_or_else(|| "Could not work out a folder name from that URL".to_string())?;

    let parent = Path::new(dest_parent);
    if !parent.is_dir() {
        return Err(format!("{dest_parent} is not a folder"));
    }
    let dest = parent.join(&name);
    if dest.exists() {
        return Err(format!("{} already exists", dest.display()));
    }

    let output = git()
        .arg("clone")
        .arg("--")
        .arg(url)
        .arg(&dest)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Could not run git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("git clone failed");
        return Err(reason.trim().to_string());
    }
    Ok(dest.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_ahead_behind_and_clean_state() {
        let out = "# branch.oid 1234567890abcdef\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +2 -1\n";
        let info = parse_status(out).unwrap();
        assert_eq!(info, GitInfo { branch: "main".into(), dirty: false, ahead: 2, behind: 1 });
    }

    #[test]
    fn any_changed_entry_marks_the_repo_dirty() {
        let out = "# branch.oid abc\n# branch.head dev\n1 .M N... 100644 100644 100644 aaa bbb src/main.rs\n";
        let info = parse_status(out).unwrap();
        assert!(info.dirty);
        assert_eq!((info.ahead, info.behind), (0, 0));
    }

    #[test]
    fn detached_head_shows_short_sha() {
        let out = "# branch.oid 0123456789abcdef\n# branch.head (detached)\n";
        assert_eq!(parse_status(out).unwrap().branch, "detached@0123456");
    }

    #[test]
    fn output_without_branch_header_is_not_a_repo() {
        assert_eq!(parse_status(""), None);
        assert_eq!(parse_status("fatal: not a git repository"), None);
    }

    #[test]
    fn repo_names_from_common_url_shapes() {
        let cases = [
            ("https://github.com/edu526/vori.git", "vori"),
            ("https://github.com/edu526/vori", "vori"),
            ("https://github.com/edu526/vori/", "vori"),
            ("git@github.com:edu526/vori.git", "vori"),
            ("ssh://git@host:2222/team/app.git", "app"),
            ("/home/me/repos/thing", "thing"),
        ];
        for (url, want) in cases {
            assert_eq!(repo_name_from_url(url).as_deref(), Some(want), "{url}");
        }
        assert_eq!(repo_name_from_url(""), None);
        assert_eq!(repo_name_from_url("https://host/.."), None);
    }

    #[test]
    fn clone_rejects_option_like_and_blank_urls_before_running_git() {
        assert!(clone("--upload-pack=evil", ".").is_err());
        assert!(clone("   ", ".").is_err());
        assert!(clone("https://a/b c.git", ".").is_err());
    }

    #[test]
    fn reads_a_real_repository() {
        if Command::new("git").arg("--version").output().is_err() {
            return; // git not installed: nothing to test against
        }
        let dir = std::env::temp_dir().join(format!("vori-git-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert_eq!(read_info(dir.to_str().unwrap()), None, "plain folder is not a repo");

        assert!(Command::new("git").arg("init").arg("-q").arg(&dir).status().unwrap().success());
        let info = read_info(dir.to_str().unwrap()).expect("initialised repo");
        assert!(!info.branch.is_empty());
        assert!(!info.dirty);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
