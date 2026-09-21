use std::collections::{HashMap, HashSet};

use crate::models::{category::CategoriesMap, project::ProjectsMap};

/// Environment variable Claude Code reads to locate its config dir (login, history, settings).
pub const CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";

/// Expand a leading `~` to the user's home directory.
pub fn expand_home(path: &str) -> String {
    let rest = match path.strip_prefix('~') {
        Some(r) if r.is_empty() || r.starts_with('/') || r.starts_with('\\') => r,
        _ => return path.to_string(),
    };
    match dirs::home_dir() {
        Some(home) => format!("{}{}", home.display(), rest),
        None => path.to_string(),
    }
}

fn normalize(path: &str) -> String {
    let unified = path.replace('\\', "/");
    let trimmed = unified.trim_end_matches('/');
    let base = if trimmed.is_empty() { "/" } else { trimmed };
    if cfg!(windows) {
        base.to_lowercase()
    } else {
        base.to_string()
    }
}

/// True when `path` is `candidate` itself or lives inside it.
fn is_within(path: &str, candidate: &str) -> bool {
    path == candidate
        || path.starts_with(&format!("{}/", candidate.trim_end_matches('/')))
}

/// First profile found walking from `category` up through its ancestors.
fn profile_from_category_chain(category: &str, categories: &CategoriesMap) -> Option<String> {
    let mut visited = HashSet::new();
    let mut current = Some(category.to_string());
    while let Some(key) = current {
        // Guard against cycles in hand-edited config
        if !visited.insert(key.clone()) {
            return None;
        }
        let cat = categories.get(&key)?;
        if let Some(profile) = &cat.claude_profile {
            return Some(profile.clone());
        }
        current = cat.parent.clone();
    }
    None
}

/// Name of the Claude profile that applies to `path`.
///
/// The most specific registered node wins: a project whose path contains `path`,
/// or a category whose bound folder contains it. A project's own profile
/// overrides its categories; otherwise the nearest ancestor category with one wins.
pub fn resolve_profile_name(
    path: &str,
    categories: &CategoriesMap,
    projects: &ProjectsMap,
) -> Option<String> {
    let target = normalize(path);
    // (candidate path length, resolved profile) of the best match so far
    let mut best: Option<(usize, Option<String>)> = None;

    // Sorted so ties between equal paths resolve deterministically
    let mut project_keys: Vec<&String> = projects.keys().collect();
    project_keys.sort();
    for key in project_keys {
        let project = &projects[key];
        let candidate = normalize(&project.path);
        if is_within(&target, &candidate) && best.as_ref().map_or(true, |(len, _)| candidate.len() > *len) {
            let profile = project
                .claude_profile
                .clone()
                .or_else(|| profile_from_category_chain(&project.parent, categories));
            best = Some((candidate.len(), profile));
        }
    }

    let mut category_keys: Vec<&String> = categories.keys().collect();
    category_keys.sort();
    for key in category_keys {
        let Some(source) = categories[key].source_path.as_deref() else { continue };
        let candidate = normalize(source);
        if is_within(&target, &candidate) && best.as_ref().map_or(true, |(len, _)| candidate.len() > *len) {
            best = Some((candidate.len(), profile_from_category_chain(key, categories)));
        }
    }

    best.and_then(|(_, profile)| profile)
}

/// `CLAUDE_CONFIG_DIR` value to use when opening `path`, or `None` when no profile applies.
pub fn config_dir_for_path(
    path: &str,
    categories: &CategoriesMap,
    projects: &ProjectsMap,
    profiles: &HashMap<String, String>,
) -> Option<String> {
    let name = resolve_profile_name(path, categories, projects)?;
    let dir = profiles.get(&name)?;
    Some(expand_home(dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{category::Category, project::Project};

    fn cat(parent: Option<&str>, source: Option<&str>, profile: Option<&str>) -> Category {
        Category {
            parent: parent.map(String::from),
            source_path: source.map(String::from),
            claude_profile: profile.map(String::from),
        }
    }

    fn proj(path: &str, parent: &str, profile: Option<&str>) -> Project {
        Project {
            path: path.to_string(),
            parent: parent.to_string(),
            stack: None,
            claude_profile: profile.map(String::from),
        }
    }

    fn fixture() -> (CategoriesMap, ProjectsMap) {
        let mut cats = CategoriesMap::new();
        cats.insert("work".into(), cat(None, Some("/code/work"), Some("trabajo")));
        cats.insert("work/api".into(), cat(Some("work"), None, None));
        cats.insert("personal".into(), cat(None, Some("/code/personal"), None));
        cats.insert("oss".into(), cat(Some("work"), None, Some("oss")));

        let mut projs = ProjectsMap::new();
        projs.insert("billing".into(), proj("/code/work/billing", "work/api", None));
        projs.insert("side".into(), proj("/code/personal/side", "personal", None));
        projs.insert("fork".into(), proj("/code/work/fork", "oss", None));
        projs.insert("special".into(), proj("/code/work/special", "work/api", Some("personal")));
        (cats, projs)
    }

    #[test]
    fn project_inherits_from_nearest_ancestor_category() {
        let (cats, projs) = fixture();
        // work/api has none → falls through to work
        assert_eq!(resolve_profile_name("/code/work/billing", &cats, &projs).as_deref(), Some("trabajo"));
        // oss overrides work
        assert_eq!(resolve_profile_name("/code/work/fork", &cats, &projs).as_deref(), Some("oss"));
    }

    #[test]
    fn project_profile_overrides_categories() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/work/special", &cats, &projs).as_deref(), Some("personal"));
    }

    #[test]
    fn no_profile_anywhere_resolves_to_none() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/personal/side", &cats, &projs), None);
    }

    #[test]
    fn unregistered_path_uses_enclosing_category_folder() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/work/new-thing", &cats, &projs).as_deref(), Some("trabajo"));
    }

    #[test]
    fn path_inside_a_project_uses_that_project() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/work/special/src/lib", &cats, &projs).as_deref(), Some("personal"));
    }

    #[test]
    fn sibling_folder_with_shared_prefix_does_not_match() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/workshop/x", &cats, &projs), None);
    }

    #[test]
    fn trailing_separators_and_backslashes_are_ignored() {
        let (cats, projs) = fixture();
        assert_eq!(resolve_profile_name("/code/work/billing/", &cats, &projs).as_deref(), Some("trabajo"));
        assert_eq!(resolve_profile_name("\\code\\work\\billing", &cats, &projs).as_deref(), Some("trabajo"));
    }

    #[test]
    fn category_cycle_does_not_hang() {
        let mut cats = CategoriesMap::new();
        cats.insert("a".into(), cat(Some("b"), None, None));
        cats.insert("b".into(), cat(Some("a"), None, None));
        let mut projs = ProjectsMap::new();
        projs.insert("p".into(), proj("/x/p", "a", None));
        assert_eq!(resolve_profile_name("/x/p", &cats, &projs), None);
    }

    #[test]
    fn unknown_profile_name_yields_no_config_dir() {
        let (cats, projs) = fixture();
        let profiles = HashMap::new();
        assert_eq!(config_dir_for_path("/code/work/billing", &cats, &projs, &profiles), None);
    }

    #[test]
    fn config_dir_is_returned_for_known_profile() {
        let (cats, projs) = fixture();
        let mut profiles = HashMap::new();
        profiles.insert("trabajo".to_string(), "/home/u/.claude-work".to_string());
        assert_eq!(
            config_dir_for_path("/code/work/billing", &cats, &projs, &profiles).as_deref(),
            Some("/home/u/.claude-work")
        );
    }

    #[test]
    fn tilde_is_expanded_only_as_a_path_prefix() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_home("~/.claude-work"), format!("{}/.claude-work", home.display()));
        assert_eq!(expand_home("~"), home.display().to_string());
        assert_eq!(expand_home("~other/x"), "~other/x");
        assert_eq!(expand_home("/abs/~/x"), "/abs/~/x");
    }
}
