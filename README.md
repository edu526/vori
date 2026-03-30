# Vori

A Finder-style project launcher for VSCode, Kiro, and other editors. Browse your categories and projects in a column browser, open them directly in your editor or terminal, and manage multi-project workspaces — all from a lightweight tray app.

Cross-platform: Linux, macOS, Windows.

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

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
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
npm install
npm run tauri build
```

Binaries will be in `src-tauri/target/release/bundle/`.

### Development

```bash
npm run tauri dev   # hot reload
npm run dev         # frontend only (no Rust, faster)
npm run check       # type check
```

---

## Configuration

Config lives in the OS user config directory under `code-launcher/` (compatible with v1):

| OS | Path |
|----|------|
| Linux | `~/.config/code-launcher/` |
| macOS | `~/Library/Application Support/code-launcher/` |
| Windows | `%APPDATA%\code-launcher\` |

Files: `categories.json`, `projects.json`, `files.json`, `preferences.json`, `favorites.json`, `recents.json`.

---

## Tech stack

- [Tauri 2](https://tauri.app/) — native shell
- [Svelte 5](https://svelte.dev/) + TypeScript — UI
- [Rust](https://www.rust-lang.org/) — backend / IPC / file I/O
