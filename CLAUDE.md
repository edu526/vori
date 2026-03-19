# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this

**Vori** is a cross-platform (Linux, macOS, Windows) rebuild of [code-launcher](../code-launcher) — a Finder-style project launcher for VSCode and Kiro editors. The original was Python/GTK (Linux-only). This version is Tauri 2 + Svelte 5 + TypeScript.

## Commands

```bash
# Prerequisites (Linux only, run once)
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf \
  libayatana-appindicator3-dev

# Install JS dependencies
npm install

# Development (hot reload)
npm run tauri dev

# Production build
npm run tauri build

# Frontend only (no Rust, faster iteration on UI)
npm run dev

# Type check
npm run check
```

## Planned architecture

### Layer structure

```
src/                          # Frontend — Svelte 5 + TypeScript
  lib/
    components/
      columns/                # ColumnBrowser, Column, ColumnItem
      context-menu/           # ContextMenu component
      dialogs/                # CategoryDialog, ProjectDialog, PreferencesDialog
      ui/                     # Primitives (shadcn-svelte)
    stores/
      navigation.svelte.ts    # Column stack state (Svelte 5 runes)
      config.svelte.ts        # Categories / projects / files reactive state
      theme.svelte.ts         # OS detection → CSS theme switching
    api/
      commands.ts             # Type-safe wrappers around invoke()
      types.ts                # TypeScript types mirroring Rust models
  routes/
    +page.svelte              # Single-page app shell
  app.css                     # Global styles
  themes/
    macos.css                 # -apple-system font, blur, HIG colors
    windows.css               # Fluent Design tokens
    linux.css                 # Adwaita-inspired tokens

src-tauri/                    # Backend — Rust
  src/
    lib.rs                    # App setup, plugin registration, AppState
    main.rs                   # Entry point
    commands/
      mod.rs
      config.rs               # CRUD for categories, projects, files, preferences
      launcher.rs             # Open project in editor / open terminal
      search.rs               # Search logic
    models/
      mod.rs
      category.rs
      project.rs
      preferences.rs
    services/
      config_manager.rs       # JSON persistence (serde_json), uses `dirs` crate for paths
      terminal.rs             # Terminal detection per OS
      editor.rs               # VSCode / Kiro launch wrappers
    state.rs                  # AppState struct with Mutex-wrapped config
```

### Key design decisions

- **Single source of truth in Rust**: config state lives in `AppState` (Rust), frontend is read-only view. No duplicating state between Rust and Svelte stores.
- **IPC via typed commands**: all Tauri commands return `Result<T, String>`; frontend handles errors uniformly via `commands.ts` wrappers.
- **Svelte 5 runes**: use `$state`, `$derived`, `$effect` — no legacy stores.
- **OS theming**: detect platform at runtime with `@tauri-apps/plugin-os`, apply a CSS class on `<body>` (`os-macos` / `os-windows` / `os-linux`), each maps to a theme file.
- **Single-instance**: `tauri-plugin-single-instance` — replaces the Python `fcntl` + SIGUSR1 pattern.
- **Config paths**: `dirs` crate — maps to `~/.config/code-launcher` on Linux, `~/Library/Application Support/code-launcher` on macOS, `%APPDATA%/code-launcher` on Windows. El directorio se mantiene como `code-launcher` para compatibilidad total con v1.

### Data model (unchanged from v1, backward-compatible)

Config lives in the OS user config dir under `code-launcher/` (compatible with v1):
- `categories.json` — hierarchy, **hard limit of 2 levels** (Category → Subcategory)
- `projects.json` — project entries mapped to categories
- `files.json` — individual files for text editing
- `preferences.json` — default editor, terminal, close-on-open
- `favorites.json` / `recents.json`

### Rust crates (planned)

```toml
serde = { features = ["derive"] }
serde_json = "*"
dirs = "*"                          # platform config paths
tauri-plugin-single-instance = "*"
tauri-plugin-os = "*"
tauri-plugin-shell = "*"            # launch editors and terminals
tauri-plugin-dialog = "*"           # folder picker
```

### Frontend packages (planned)

```
shadcn-svelte      # headless accessible components
@tauri-apps/api
@tauri-apps/plugin-os
@tauri-apps/plugin-shell
```
