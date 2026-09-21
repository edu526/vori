//! `package.json` scripts a project can run from Vori's context menu.
//!
//! The frontend never supplies a command: it names a script, and the backend re-reads
//! `package.json`, checks the script exists and builds `<manager> run <script>` itself. Script
//! names are also restricted to a plain charset, so nothing in a (possibly untrusted) repo can
//! smuggle shell syntax into the command line.

use std::path::Path;

use serde::Serialize;

/// Shown first, in this order.
const PRIORITY: &[&str] = &["dev", "start", "serve", "build", "test", "lint"];
/// npm lifecycle hooks that aren't something a person runs by hand.
const LIFECYCLE: &[&str] = &[
    "install", "prepare", "prepublish", "prepublishOnly", "prepack", "postpack", "publish",
];
const MAX_SCRIPTS: usize = 12;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectScripts {
    /// `npm`, `pnpm`, `yarn` or `bun`.
    pub manager: String,
    pub scripts: Vec<String>,
}

fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-'))
}

/// `packageManager` field (`"pnpm@10.1.0"`), else the lockfile, else npm.
fn detect_manager(dir: &Path, package: &serde_json::Value) -> &'static str {
    if let Some(field) = package.get("packageManager").and_then(|v| v.as_str()) {
        match field.split('@').next().unwrap_or("") {
            "pnpm" => return "pnpm",
            "yarn" => return "yarn",
            "bun" => return "bun",
            "npm" => return "npm",
            _ => {}
        }
    }
    let has = |f: &str| dir.join(f).exists();
    if has("pnpm-lock.yaml") {
        "pnpm"
    } else if has("yarn.lock") {
        "yarn"
    } else if has("bun.lockb") || has("bun.lock") {
        "bun"
    } else {
        "npm"
    }
}

fn read_package(dir: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(dir.join("package.json")).ok()?;
    serde_json::from_str(&text).ok()
}

/// Runnable scripts of the Node project in `dir`, or `None` if it isn't one / has none.
pub fn list(dir: &Path) -> Option<ProjectScripts> {
    let package = read_package(dir)?;
    let all: Vec<&String> = package.get("scripts")?.as_object()?.keys().collect();

    let mut scripts: Vec<String> = all
        .iter()
        .filter(|name| is_safe_name(name))
        .filter(|name| !LIFECYCLE.contains(&name.as_str()))
        // `pretest` / `postbuild` run automatically around `test` / `build` (a script of the
        // project, or one of npm's own such as `install`)
        .filter(|name| {
            !["pre", "post"].iter().any(|p| {
                name.strip_prefix(p).is_some_and(|base| {
                    LIFECYCLE.contains(&base) || all.iter().any(|other| other.as_str() == base)
                })
            })
        })
        .map(|name| name.to_string())
        .collect();

    scripts.sort_by_key(|name| {
        (
            PRIORITY.iter().position(|p| p == name).unwrap_or(PRIORITY.len()),
            name.to_lowercase(),
        )
    });
    scripts.truncate(MAX_SCRIPTS);

    if scripts.is_empty() {
        return None;
    }
    Some(ProjectScripts { manager: detect_manager(dir, &package).to_string(), scripts })
}

/// The command line for `script`, only if it is one of the project's runnable scripts.
pub fn command_for(dir: &Path, script: &str) -> Result<String, String> {
    let found = list(dir).ok_or_else(|| "This project has no runnable scripts".to_string())?;
    if !found.scripts.iter().any(|s| s == script) {
        return Err(format!("This project has no script called “{script}”"));
    }
    Ok(format!("{} run {}", found.manager, script))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str, package_json: &str, lockfile: Option<&str>) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("vori-scripts-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), package_json).unwrap();
        if let Some(l) = lockfile {
            std::fs::write(dir.join(l), "").unwrap();
        }
        dir
    }

    #[test]
    fn common_scripts_come_first_and_hooks_are_hidden() {
        let dir = project(
            "order",
            r#"{"scripts":{"zeta":"x","build":"x","dev":"x","prebuild":"x","postinstall":"x","test":"x","alpha":"x"}}"#,
            None,
        );
        let found = list(&dir).unwrap();
        assert_eq!(found.scripts, ["dev", "build", "test", "alpha", "zeta"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_pre_script_without_a_base_script_is_kept() {
        let dir = project("pre", r#"{"scripts":{"prettier":"x","predeploy":"x"}}"#, None);
        // "predeploy" has no "deploy" to hook into, and "prettier" isn't pre+"ttier"
        assert_eq!(list(&dir).unwrap().scripts, ["predeploy", "prettier"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn unsafe_script_names_are_never_offered() {
        let dir = project(
            "unsafe",
            r#"{"scripts":{"dev":"x","x & calc":"x","a;b":"x","$(id)":"x","--flag":"x","ok:name-1.2":"x"}}"#,
            None,
        );
        assert_eq!(list(&dir).unwrap().scripts, ["dev", "ok:name-1.2"]);
        assert!(command_for(&dir, "x & calc").is_err());
        assert!(command_for(&dir, "--flag").is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn package_manager_field_beats_lockfile_beats_default() {
        let a = project("pm-field", r#"{"packageManager":"pnpm@10.1.0","scripts":{"dev":"x"}}"#, Some("yarn.lock"));
        assert_eq!(list(&a).unwrap().manager, "pnpm");
        let b = project("pm-lock", r#"{"scripts":{"dev":"x"}}"#, Some("yarn.lock"));
        assert_eq!(list(&b).unwrap().manager, "yarn");
        let c = project("pm-bun", r#"{"scripts":{"dev":"x"}}"#, Some("bun.lock"));
        assert_eq!(list(&c).unwrap().manager, "bun");
        let d = project("pm-none", r#"{"scripts":{"dev":"x"}}"#, None);
        assert_eq!(list(&d).unwrap().manager, "npm");
        for dir in [a, b, c, d] {
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn command_is_built_only_for_existing_scripts() {
        let dir = project("cmd", r#"{"scripts":{"dev":"vite"}}"#, Some("pnpm-lock.yaml"));
        assert_eq!(command_for(&dir, "dev").unwrap(), "pnpm run dev");
        assert!(command_for(&dir, "build").unwrap_err().contains("no script"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn non_node_or_scriptless_folders_have_nothing_to_list() {
        let empty = std::env::temp_dir().join(format!("vori-scripts-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(list(&empty), None);
        let none = project("none", r#"{"name":"x"}"#, None);
        assert_eq!(list(&none), None);
        let broken = project("broken", "{ not json", None);
        assert_eq!(list(&broken), None);
        for dir in [empty, none, broken] {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}
