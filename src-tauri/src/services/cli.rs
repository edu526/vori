//! Command line and `vori://` deep link handling.
//!
//! ```text
//! vori .                 reveal this folder in Vori, or offer to add it as a project
//! vori add [path]        same, `path` defaults to the current folder
//! vori open <name>       open the project called <name> in the default editor
//! vori vori://open/<name>   the same, as a link (`vori://open/my%20app`)
//! ```
//! On Windows and Linux the OS hands a deep link to the app as a plain argument, so links
//! and CLI arguments share one parser.

use std::path::Path;

use serde::Serialize;

use crate::models::project::ProjectsMap;
use crate::services::claude_profile::normalize;

pub const SCHEME: &str = "vori";

#[derive(Debug, Clone, PartialEq)]
pub enum CliAction {
    /// Nothing actionable: just bring the window up (or toggle it).
    Show,
    /// Reveal the folder if it is already a project, otherwise offer to add it.
    AddOrReveal { path: String },
    /// Open the named project in the default editor.
    Open { name: String },
}

/// What the running UI has to do. Queued until the frontend asks for it, because a first
/// launch (`vori .` with the app closed) can run before the page has loaded.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FrontendRequest {
    /// `existing` is the key of the project that already has this path.
    AddOrReveal { path: String, existing: Option<String> },
    /// A project was opened from outside the UI; refresh the recents list.
    Opened { path: String, name: String },
    /// Nothing matches the project name that was asked for.
    NoProject { query: String },
    /// Several projects match the name that was asked for.
    AmbiguousProject { query: String, matches: Vec<String> },
    /// Anything else that went wrong: `message` comes straight from the failing step.
    Failed { message: String },
}

/// Why a project name could not be resolved to a single project.
#[derive(Debug, Clone, PartialEq)]
pub enum FindError {
    NoMatch,
    Several(Vec<String>),
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(v) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Absolute, `\\?\`-free version of `arg` (relative to `cwd`) if it is an existing folder.
fn existing_dir(arg: &str, cwd: &str) -> Option<String> {
    let path = Path::new(arg);
    let joined = if path.is_absolute() { path.to_path_buf() } else { Path::new(cwd).join(path) };
    let canonical = std::fs::canonicalize(joined).ok()?;
    if !canonical.is_dir() {
        return None;
    }
    let s = canonical.to_string_lossy().into_owned();
    Some(s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s))
}

fn parse_link(url: &str) -> CliAction {
    let rest = &url[SCHEME.len() + 3..]; // caller checked the "vori://" prefix
    let rest = rest.split(['?', '#']).next().unwrap_or("");
    match rest.split_once('/') {
        Some(("open", name)) if !name.trim_matches('/').is_empty() => {
            CliAction::Open { name: percent_decode(name.trim_matches('/')) }
        }
        _ => CliAction::Show,
    }
}

/// Interpret arguments (without the program name).
pub fn parse(args: &[String], cwd: &str) -> CliAction {
    let mut positional = args.iter().filter(|a| !a.starts_with("--"));
    let Some(first) = positional.next() else {
        return CliAction::Show;
    };

    if first.len() > SCHEME.len() + 3
        && first[..SCHEME.len() + 3].eq_ignore_ascii_case(&format!("{SCHEME}://"))
    {
        return parse_link(first);
    }

    match first.as_str() {
        "open" => match positional.next() {
            Some(name) if !name.trim().is_empty() => CliAction::Open { name: name.trim().to_string() },
            _ => CliAction::Show,
        },
        "add" => {
            let target = positional.next().map(String::as_str).unwrap_or(".");
            match existing_dir(target, cwd) {
                Some(path) => CliAction::AddOrReveal { path },
                None => CliAction::Show,
            }
        }
        other => match existing_dir(other, cwd) {
            Some(path) => CliAction::AddOrReveal { path },
            None => CliAction::Show,
        },
    }
}

/// Project whose key matches `query`: exact, then case-insensitive, then a unique
/// case-insensitive prefix, then a unique substring.
pub fn find_project(projects: &ProjectsMap, query: &str) -> Result<(String, String), FindError> {
    let q = query.trim();
    if let Some(p) = projects.get(q) {
        return Ok((q.to_string(), p.path.clone()));
    }
    let lower = q.to_lowercase();

    let tiers: [Box<dyn Fn(&str) -> bool>; 3] = [
        Box::new(|k| k.to_lowercase() == lower),
        Box::new(|k| k.to_lowercase().starts_with(&lower)),
        Box::new(|k| k.to_lowercase().contains(&lower)),
    ];
    for matches in tiers.iter() {
        let mut hits: Vec<&String> = projects.keys().filter(|k| matches(k.as_str())).collect();
        hits.sort();
        match hits.as_slice() {
            [] => continue,
            [one] => return Ok(((*one).clone(), projects[one.as_str()].path.clone())),
            many => {
                return Err(FindError::Several(many.iter().take(5).map(|k| k.to_string()).collect()));
            }
        }
    }
    Err(FindError::NoMatch)
}

/// Key of the project that lives at `path`, comparing paths loosely (slashes, case on Windows).
pub fn project_at_path(projects: &ProjectsMap, path: &str) -> Option<String> {
    let wanted = normalize(path);
    projects
        .iter()
        .find(|(_, p)| normalize(&p.path) == wanted)
        .map(|(k, _)| k.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::Project;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn projects(keys: &[&str]) -> ProjectsMap {
        keys.iter()
            .map(|k| {
                (
                    k.to_string(),
                    Project { path: format!("/work/{k}"), parent: "c".into(), stack: None, claude_profile: None },
                )
            })
            .collect()
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("vori-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn no_arguments_or_only_flags_just_show_the_window() {
        assert_eq!(parse(&s(&[]), "."), CliAction::Show);
        assert_eq!(parse(&s(&["--autostart"]), "."), CliAction::Show);
    }

    #[test]
    fn open_takes_the_project_name() {
        assert_eq!(parse(&s(&["open", "vori"]), "."), CliAction::Open { name: "vori".into() });
        assert_eq!(parse(&s(&["open"]), "."), CliAction::Show);
    }

    #[test]
    fn deep_links_are_parsed_and_percent_decoded() {
        assert_eq!(parse(&s(&["vori://open/my%20app"]), "."), CliAction::Open { name: "my app".into() });
        assert_eq!(parse(&s(&["VORI://open/x/"]), "."), CliAction::Open { name: "x".into() });
        assert_eq!(parse(&s(&["vori://open/x?ref=1"]), "."), CliAction::Open { name: "x".into() });
        assert_eq!(parse(&s(&["vori://open/"]), "."), CliAction::Show);
        assert_eq!(parse(&s(&["vori://whatever/x"]), "."), CliAction::Show);
        assert_eq!(parse(&s(&["vori://"]), "."), CliAction::Show);
    }

    #[test]
    fn folders_resolve_against_the_working_directory() {
        let base = temp_dir("dirs");
        std::fs::create_dir_all(base.join("app")).unwrap();
        let cwd = base.to_string_lossy().into_owned();

        let expected = std::fs::canonicalize(base.join("app")).unwrap();
        let expected = expected.to_string_lossy().trim_start_matches(r"\\?\").to_string();

        assert_eq!(parse(&s(&["app"]), &cwd), CliAction::AddOrReveal { path: expected.clone() });
        assert_eq!(parse(&s(&["add", "app"]), &cwd), CliAction::AddOrReveal { path: expected });
        // `.` is the working directory itself
        assert!(matches!(parse(&s(&["."]), &cwd), CliAction::AddOrReveal { .. }));
        assert!(matches!(parse(&s(&["add"]), &cwd), CliAction::AddOrReveal { .. }));
        // things that aren't folders are ignored
        assert_eq!(parse(&s(&["nope"]), &cwd), CliAction::Show);
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn project_lookup_prefers_exact_then_unique_partial_matches() {
        let p = projects(&["vori", "Vori-docs", "api", "api-gateway"]);
        assert_eq!(find_project(&p, "vori").unwrap().0, "vori"); // exact beats prefix
        assert_eq!(find_project(&p, "VORI-DOCS").unwrap().0, "Vori-docs");
        assert_eq!(find_project(&p, "gate").unwrap().0, "api-gateway"); // unique substring
        assert!(find_project(&p, "api").unwrap().0 == "api"); // exact
        // ambiguous prefix: both "vori" and "Vori-docs" start with "vo"
        assert_eq!(find_project(&p, "vo").unwrap_err(), FindError::Several(vec!["Vori-docs".into(), "vori".into()]));
        assert_eq!(find_project(&p, "zzz").unwrap_err(), FindError::NoMatch);
    }

    #[test]
    fn project_at_path_ignores_slash_style_and_trailing_separator() {
        let mut p = projects(&["vori"]);
        p.get_mut("vori").unwrap().path = "/work/vori/".into();
        assert_eq!(project_at_path(&p, "/work/vori").as_deref(), Some("vori"));
        assert_eq!(project_at_path(&p, "\\work\\vori").as_deref(), Some("vori"));
        assert_eq!(project_at_path(&p, "/work/other"), None);
    }

    #[test]
    fn percent_decode_handles_edges() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("caf%C3%A9"), "café");
    }
}
