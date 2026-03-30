#!/bin/sh
set -e

REPO="edu526/vori"
API="https://api.github.com/repos/$REPO/releases/latest"

# ── helpers ──────────────────────────────────────────────────────────────────

info()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()    { printf '\033[1;32m  ✓\033[0m %s\n' "$*"; }
die()   { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed."; }

need curl
need uname

# ── detect OS / arch ─────────────────────────────────────────────────────────

OS=$(uname -s)
ARCH=$(uname -m)

case "$ARCH" in
  x86_64)         ARCH_TAG="x86_64" ;;
  aarch64|arm64)  ARCH_TAG="aarch64" ;;
  *)              die "Unsupported architecture: $ARCH" ;;
esac

# ── fetch latest release ──────────────────────────────────────────────────────

info "Fetching latest release from $REPO..."
RELEASE=$(curl -sf "$API") || die "Could not fetch release info. Check your internet connection."
VERSION=$(printf '%s' "$RELEASE" | grep -o '"tag_name": *"[^"]*"' | grep -o '"v[^"]*"' | tr -d '"')
info "Latest version: $VERSION"

asset_url() {
  printf '%s' "$RELEASE" | grep -o '"browser_download_url": *"[^"]*'"$1"'[^"]*"' | grep -o 'https://[^"]*' | head -1
}

# ── install ───────────────────────────────────────────────────────────────────

case "$OS" in
  Linux)
    if [ -f /etc/debian_version ]; then
      URL=$(asset_url '\.deb"')
      [ -z "$URL" ] && die "No .deb asset found in latest release."
      TMP=$(mktemp /tmp/vori.XXXXXX.deb)
      info "Downloading $URL..."
      curl -sfL "$URL" -o "$TMP"
      info "Installing .deb package..."
      sudo dpkg -i "$TMP"
      rm -f "$TMP"

    elif [ -f /etc/redhat-release ] || [ -f /etc/fedora-release ] || [ -f /etc/os-release ] && grep -q 'ID_LIKE=.*rhel\|ID=fedora\|ID=rhel' /etc/os-release 2>/dev/null; then
      URL=$(asset_url '\.rpm"')
      [ -z "$URL" ] && die "No .rpm asset found in latest release."
      TMP=$(mktemp /tmp/vori.XXXXXX.rpm)
      info "Downloading $URL..."
      curl -sfL "$URL" -o "$TMP"
      info "Installing .rpm package..."
      sudo rpm -i "$TMP"
      rm -f "$TMP"

    else
      URL=$(asset_url '\.AppImage"')
      [ -z "$URL" ] && die "No .AppImage asset found in latest release."
      DEST="$HOME/.local/bin/vori"
      mkdir -p "$HOME/.local/bin"
      info "Downloading AppImage to $DEST..."
      curl -sfL "$URL" -o "$DEST"
      chmod +x "$DEST"
      ok "Installed to $DEST"
      printf '\n  Make sure %s is in your PATH.\n\n' "$HOME/.local/bin"
    fi
    ;;

  Darwin)
    URL=$(printf '%s' "$RELEASE" | grep -o '"browser_download_url": *"[^"]*'"$ARCH_TAG"'[^"]*\.dmg"' | grep -o 'https://[^"]*' | head -1)
    [ -z "$URL" ] && URL=$(asset_url '\.dmg"')
    [ -z "$URL" ] && die "No .dmg asset found in latest release."

    TMP=$(mktemp /tmp/vori.XXXXXX.dmg)
    info "Downloading $URL..."
    curl -sfL "$URL" -o "$TMP"

    info "Mounting disk image..."
    hdiutil attach "$TMP" -mountpoint /tmp/vori-mount -quiet

    info "Installing Vori.app to /Applications..."
    cp -r /tmp/vori-mount/Vori.app /Applications/

    hdiutil detach /tmp/vori-mount -quiet
    rm -f "$TMP"

    # Remove quarantine so Gatekeeper doesn't block an unsigned app
    xattr -dr com.apple.quarantine /Applications/Vori.app 2>/dev/null || true
    ;;

  *)
    die "Unsupported OS: $OS. Use the Windows installer (install.ps1) on Windows."
    ;;
esac

ok "Vori $VERSION installed successfully."
