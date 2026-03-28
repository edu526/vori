use std::path::Path;

/// Returns the detected stack key for a directory, or "unknown".
pub fn detect_stack(path: &Path) -> String {
    // Priority-ordered list: (marker files, stack key)
    const MARKERS: &[(&[&str], &str)] = &[
        (&["Cargo.toml"], "rust"),
        (&["go.mod"], "go"),
        (&["pubspec.yaml"], "flutter"),
        (&["package.json"], "node"),
        (&["pyproject.toml", "setup.py", "requirements.txt"], "python"),
        (&["pom.xml", "build.gradle", "build.gradle.kts"], "java"),
        (&["composer.json"], "php"),
        (&["Gemfile"], "ruby"),
        (&["CMakeLists.txt"], "cpp"),
        (&["Makefile"], "make"),
    ];

    // Check glob extensions (.sln, .csproj) — needs dir scan
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let name = fname.to_string_lossy();
            if name.ends_with(".sln") || name.ends_with(".csproj") || name.ends_with(".fsproj") {
                return "dotnet".to_string();
            }
            if name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace") {
                return "swift".to_string();
            }
        }
    }

    for (files, stack) in MARKERS {
        if files.iter().any(|f| path.join(f).exists()) {
            return stack.to_string();
        }
    }

    // Check for hidden VCS as a weak signal (bare repo or worktree)
    if path.join(".git").exists() {
        return "git".to_string();
    }

    "unknown".to_string()
}
