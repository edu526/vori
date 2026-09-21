# Vori

A Finder-style project launcher for VSCode, Kiro, and other editors. Browse your categories and projects in a column browser, open them directly in your editor or terminal, and manage multi-project workspaces — all from a lightweight tray app.

Cross-platform: Linux, macOS, Windows.

## Features

- **Column browser** for categories (any depth), projects and files, with favorites and recents
- **Git at a glance**: branch, uncommitted-changes dot and ahead/behind on every repository, plus **Clone repository…** into a category
- **Quick actions** per project: open in any detected editor or terminal, show in the file manager, copy the path, run `package.json` scripts
- **Search** ranked by relevance and how often you open things (empty search shows your most used)
- **Folder-bound categories** flag new folders (`+N`) so you can import them
- **Multi-project workspaces**, per-category/project **Claude profiles**, in-app text editor
- **Backup**: export/import your data as one file; move the config folder with `VORI_CONFIG_DIR`
- **Command line and `vori://` links** (see below), auto-updates, and a **Spanish or English** UI (Preferences → Appearance)


---

## Installation

### One-liner install

**Linux / macOS**

```sh
curl -fsSL https://raw.githubusercontent.com/edu526/vori/main/scripts/install.sh | sh
```

**Windows** (PowerShell, run as Administrator)

```powershell
irm https://raw.githubusercontent.com/edu526/vori/main/scripts/install.ps1 | iex
```

The script auto-detects your OS and architecture, downloads the right package from the [latest release](../../releases/latest), and installs it.

---

### Manual download

Go to the [Releases](../../releases/latest) page and download the package for your OS:

| Platform | File |
|----------|------|
| Linux (Debian/Ubuntu) | `.deb` |
| Linux (Fedora/RHEL) | `.rpm` |
| Linux (universal) | `.AppImage` |
| macOS (Apple Silicon) | `.dmg` (aarch64) |
| macOS (Intel) | `.dmg` (x86_64) |
| Windows | `.msi` or `.exe` (NSIS) |

#### Linux — AppImage

```bash
chmod +x Vori_*.AppImage
./Vori_*.AppImage
```

#### Linux — .deb

```bash
sudo dpkg -i vori_*.deb
```

#### Linux — .rpm

```bash
sudo rpm -i vori-*.rpm
```

#### macOS

Open the `.dmg`, drag Vori to Applications, and launch it. If Gatekeeper blocks it (unsigned app), run:

```bash
xattr -dr com.apple.quarantine /Applications/Vori.app
```

#### Windows

Run the `.msi` installer or the `.exe` NSIS installer and follow the wizard.

---

## Command line and links

```sh
vori .                  # reveal this folder in Vori, or offer to add it as a project
vori add ~/code/app     # same, for another folder
vori open my-project    # open a project in your default editor (no window needed)
```

`open` matches the project name exactly, then ignoring case, then by a unique prefix or substring. If several projects match, Vori tells you which.

The same actions work as links, e.g. from a bookmark or a README: `vori://open/my-project` (spaces as `%20`). Links can only open a project you already added.

| OS | How to run it |
|----|---------------|
| Linux | `vori` is on your `PATH` after installing the `.deb` / `.rpm` |
| macOS | `/Applications/Vori.app/Contents/MacOS/Vori .` (alias it as `vori`) |
| Windows | add the install folder to your `PATH`. The app has no console, so results show up in the Vori window |

If Vori is already running, the command is handed to that instance.

---

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org/) 24 LTS (pinned in `.node-version`)
- [pnpm](https://pnpm.io/) (`corepack enable pnpm`)
- [Rust](https://rustup.rs/) stable

**Linux only:**

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
  patchelf libayatana-appindicator3-dev
```

### Steps

```bash
git clone https://github.com/edu526/vori
cd vori
pnpm install
pnpm tauri build
```

Binaries will be in `src-tauri/target/release/bundle/`.

### Development

```bash
pnpm tauri dev   # hot reload
pnpm dev         # frontend only (no Rust, faster)
pnpm check       # type check
```

---

## Configuration

Config lives in the OS user config directory under `vori/`:

| OS | Path |
|----|------|
| Linux | `~/.config/vori/` |
| macOS | `~/Library/Application Support/vori/` |
| Windows | `%APPDATA%\vori\` |

Configs from older `code-launcher/` installs are copied over on first launch.

Files: `categories.json`, `projects.json`, `files.json`, `preferences.json`, `favorites.json`, `recents.json`.

Other files Vori keeps there: `usage.json` (how often things are opened, for search ranking) and `backups/` (a copy of your data made before each import, newest 5 kept).

To keep the config in a synced folder (Dropbox, OneDrive…), set the `VORI_CONFIG_DIR` environment variable to it and restart Vori.

Writes are atomic. If a file can't be parsed at startup it is moved aside as `<name>.corrupt-<timestamp>.bak` (never deleted) and Vori tells you which one.

---

## Tech stack

- [Tauri 2](https://tauri.app/) — native shell
- [Svelte 5](https://svelte.dev/) + TypeScript — UI
- [Rust](https://www.rust-lang.org/) — backend / IPC / file I/O
