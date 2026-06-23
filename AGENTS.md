# AGENTS.md

Repo-specific guidance for OpenCode sessions. Verified against the codebase on 2026-06-23. Where this conflicts with `README.md`, **trust the code**, not the docs (README is stale in a few places — noted below).

## Commands

```bash
pnpm install             # install JS deps (node_modules is gitignored)
pnpm tauri dev           # full app: Vite + Rust, hot reload, opens native window
pnpm dev                 # frontend only — runs in browser WITHOUT Rust (see below)
pnpm tauri build         # production build → src-tauri/target/release/bundle/
pnpm check               # svelte-check typecheck (runs `svelte-kit sync` first)
```

There is no test suite, no lint script, no formatter config. `pnpm check` is the only verification step — run it after frontend changes. For Rust changes, `cargo check --manifest-path src-tauri/Cargo.toml`.

### Setup gotchas (Windows)

- **Rust + MSVC required** for `tauri dev`/`tauri build`: install Visual Studio 2022 Build Tools with the `VCTools` workload (MSVC + Windows SDK), then `rustup`. `cargo check` first run downloads ~440 crates and takes ~2 min.
- **Node is managed via `fnm`** and may not be on the system PATH outside this OpenCode session. If `pnpm`/`node` aren't found, run `fnm env --use-on-cd | Out-String | Invoke-Expression` first (or path the fnm dir explicitly). **Node 24 LTS** required (pinned via `.node-version`). The repo pins pnpm via the `packageManager` field in `package.json` (`pnpm@10.34.3`); enable it with `corepack enable pnpm` (or `npm i -g pnpm`).
- `tauri dev` watches `src-tauri/` and rebuilds Rust on change; Vite ignores `src-tauri/**` and handles the frontend. Strict port 1420 only under Tauri (`strictPort: isTauri` in `vite.config.js`).

## Two dev modes — important

- `pnpm dev` (no Tauri): `vite.config.js` aliases `@tauri-apps/api/*` and `@tauri-apps/plugin-dialog` to mocks in `src/lib/mocks/`. Runs in a browser at `http://localhost:1420`. Useful for fast UI iteration; any `invoke()` call hits the mock, not Rust.
- `pnpm tauri dev`: real Rust backend. The aliases are disabled (`isTauri ? {} : browserAlias`). Use this for anything touching config, launcher, search, or window behavior.

## Architecture

**Single source of truth in Rust.** `AppState` (`src-tauri/src/state.rs`) holds all config in `Mutex`-wrapped maps. The frontend loads everything via one `get_app_data()` command (`src-tauri/src/commands/config.rs:31`) and is otherwise read-only; mutations go through typed commands that update `AppState` **and** persist to JSON in the same call.

- **IPC contract**: every `#[tauri::command]` returns `Result<T, String>`. Frontend wrappers in `src/lib/api/commands.ts` are thin `invoke()` calls — keep them 1:1 with the Rust command names (snake_case, invoked as-is).
- **Tauri command names** are registered in `src-tauri/src/lib.rs:115` (`invoke_handler!`). Adding a command requires: (1) `#[tauri::command]` fn in `commands/`, (2) re-export in `commands/mod.rs`, (3) registration in `lib.rs`. Miss any and it silently fails to bind.
- **Svelte 5 runes only** (`$state`, `$derived`, `$effect`) — no legacy stores. Stores live in `src/lib/stores/*.svelte.ts`.
- **SPA**: `adapter-static` with `index.html` fallback (`svelte.config.js`). No SSR, no prerendered routes.
- **Window**: frameless (`decorations: false`) and **hidden on start** (`visible: false` in `tauri.conf.json`). The setup hook in `lib.rs:60` shows it unless launched with `--autostart`. Toggle via tray icon or global hotkey.

## Config directory — discrepancy with docs

`README.md` says config lives under `code-launcher/`. **That's stale.** `src-tauri/src/services/config_manager.rs:17` uses `vori/` as the primary dir:

| OS | Actual path |
|----|------|
| Linux | `~/.config/vori/` |
| macOS | `~/Library/Application Support/vori/` |
| Windows | `%APPDATA%\vori\` |

On first launch, `migrate_from_legacy()` copies any `code-launcher/` JSON files into `vori/` (one-way, no-op if `vori/` already exists). Don't write config tooling that targets `code-launcher/`.

Files: `categories.json`, `projects.json`, `files.json`, `preferences.json`, `favorites.json`, `recents.json`. All JSON, pretty-printed, read/written via `config_manager::load`/`save` (typed over `serde`).

## Categories are N-level, not 2-level

The old `code-launcher` docs claimed a "hard limit of 2 levels". **Stale.** The model is flat N-level: `Category { parent: Option<String> }` (`src-tauri/src/models/category.rs`), and `add_category` accepts any parent with no depth check. `migrate_to_flat_format()` (`config_manager.rs:64`) rewrites the old 2-level `subcategories` format into the flat one on startup. Don't reintroduce a depth limit unless explicitly asked.

## Global hotkey — non-fatal on registration

The default hotkey is `Super+Shift+KeyV` (`models/preferences.rs:44`). On Windows this frequently collides with other apps. **Registration errors must not crash the app** — `lib.rs:78` handles this with `if let Err(e) = ...` + `eprintln!`, not `?`. If you touch that block, preserve the non-fatal behavior: the window remains reachable via tray icon and a second launch (single-instance plugin toggles it).

Note the asymmetry: `update_preferences` (`commands/config.rs:204`) *does* propagate hotkey errors back to the frontend as `Err(String)` — that's correct, it's a user-facing action that should surface the failure.

## Window visibility — tray off

The window defaults to `visible: false` in `tauri.conf.json` and is shown in `on_page_load` (`lib.rs:112`) when the page finishes loading. Two guards prevent the app from being stranded in a "running but unreachable" state when `show_tray = false`:

1. **Autostart launches** (`lib.rs:118`): the condition to show the window is `!autostart || !show_tray`. If the user disabled the tray, autostart launches show the window directly — otherwise the app would be running invisible at login.
2. **Close handler** (`lib.rs:131`): the condition to *hide* the window on close is `keep_background && show_tray`. If the user disabled the tray, closing the window actually closes the app — otherwise the window would hide with no tray (and the hotkey may have failed to register) and the user couldn't reopen it.

Don't add similar visibility logic without considering both paths.

## UI conventions

- **UI strings are in Spanish** (e.g. tray menu "Mostrar Vori" / "Salir" in `services/window.rs`). Match this when adding user-facing text.
- **shadcn-svelte**, style `"maia"`, base color `neutral`, icon library `hugeicons` (`components.json`). CSS entry is `src/app.css`. UI primitives go in `src/lib/components/ui/`; feature components in `src/lib/components/`.
- Themes live in `src/themes/` and are applied via an `os-{macos,windows,linux}` class on `<body>` (platform detected at runtime with `@tauri-apps/plugin-os`).

## Release flow — don't edit versions manually

Pushing to `main` triggers `.github/workflows/release.yml`:

1. **semantic-release** parses **Conventional Commits** (`feat:`, `fix:`, `chore:`, `refactor:`, `style:`, `docs:`, `ci:`, optionally scoped `feat(ui):`). Use this style for all commits.
2. `scripts/bump-version.mjs` rewrites the version in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` (first `version = "..."` only). `cargo update` bumps the lockfile.
3. semantic-release commits the bump (`chore(release): x.y.z [skip ci]`) and creates a **draft** GitHub release.
4. A matrix job builds Linux/macOS/Windows binaries and uploads them to the draft, then `publish` un-drafts it.

Do not hand-edit version fields — the next release will overwrite them and the `[skip ci]` commit needs to stay clean. Commit messages that aren't Conventional Commits are ignored by the releaser.

## Scratch files — ignore

`test.rs` (repo root) and `src-tauri/src/test_api.rs` are ad-hoc scratch files, not a test harness. There is no `cargo test` suite and no `pnpm test`. Don't wire them into anything.
