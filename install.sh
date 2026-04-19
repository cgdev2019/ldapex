#!/usr/bin/env sh
# Ldapex installer for Linux and macOS.
#
# One-liner:
#   curl -fsSL https://raw.githubusercontent.com/cgdev2019/ldapex/main/install.sh | sh
#
# What it does:
#   - detects the OS and CPU architecture
#   - queries the GitHub Releases API for the latest Ldapex tag
#   - downloads the matching asset (Linux → AppImage, macOS → dmg)
#   - Linux:   places the AppImage at $PREFIX/bin/ldapex (+x) and
#              drops a .desktop file under ~/.local/share/applications
#              so it shows up in the app launcher
#   - macOS:   mounts the dmg, copies Ldapex.app to /Applications,
#              unmounts. Requires sudo if writing to /Applications.
#
# Overrides:
#   PREFIX=/custom/path     install the Linux binary here (default:
#                           $HOME/.local, create $HOME/.local/bin as
#                           needed)
#   LDAPEX_VERSION=v1.2.3   pin a specific release tag
#
# Exit code 0 on success, non-zero otherwise.

set -eu

REPO="cgdev2019/ldapex"
BIN_NAME="ldapex"
PREFIX="${PREFIX:-$HOME/.local}"

log() { printf '\033[1;34m→\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }
die() {
  printf '\033[1;31m✗\033[0m %s\n' "$*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || die "curl is required"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  OS_KEY="linux" ;;
  Darwin) OS_KEY="macos" ;;
  *) die "unsupported OS: $OS (use install.ps1 on Windows)" ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_KEY="x86_64" ;;
  arm64|aarch64) ARCH_KEY="aarch64" ;;
  *) die "unsupported CPU architecture: $ARCH" ;;
esac

# ---- resolve the release tag ----------------------------------------

if [ -n "${LDAPEX_VERSION:-}" ]; then
  TAG="$LDAPEX_VERSION"
else
  log "Resolving the latest Ldapex release…"
  TAG="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
fi
[ -n "$TAG" ] || die "no release found yet — run \`cargo tauri build\` locally or wait for v0.1.0"
log "Installing Ldapex $TAG for $OS_KEY/$ARCH_KEY"

# ---- find the right asset URL ---------------------------------------

ASSETS_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/tags/$TAG")"

# Extract URLs + filenames from the JSON without jq (portable).
ASSETS="$(printf '%s' "$ASSETS_JSON" \
  | tr ',' '\n' \
  | sed -n 's/.*"browser_download_url": *"\([^"]*\)".*/\1/p')"

pick_asset() {
  pattern="$1"
  printf '%s\n' "$ASSETS" | grep -E "$pattern" | head -1 || true
}

case "$OS_KEY" in
  linux)
    ASSET_URL="$(pick_asset '\.AppImage$')"
    [ -n "$ASSET_URL" ] || die "no .AppImage asset in release $TAG"
    ;;
  macos)
    ASSET_URL="$(pick_asset "$ARCH_KEY.*\.dmg$")"
    [ -n "$ASSET_URL" ] || ASSET_URL="$(pick_asset '\.dmg$')"
    [ -n "$ASSET_URL" ] || die "no .dmg asset in release $TAG"
    ;;
esac

log "Downloading $(basename "$ASSET_URL")"
TMP="$(mktemp -d 2>/dev/null || mktemp -d -t ldapex)"
trap 'rm -rf "$TMP"' EXIT INT TERM
curl -fL --progress-bar -o "$TMP/payload" "$ASSET_URL"

# ---- install --------------------------------------------------------

install_linux() {
  BIN_DIR="$PREFIX/bin"
  APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
  mkdir -p "$BIN_DIR" "$APPS_DIR"

  DEST="$BIN_DIR/$BIN_NAME"
  cp "$TMP/payload" "$DEST"
  chmod +x "$DEST"
  log "Installed binary at $DEST"

  DESKTOP="$APPS_DIR/ldapex.desktop"
  cat > "$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=Ldapex
Comment=LDAP directory browser
Exec=$DEST
Icon=ldapex
Categories=Network;Utility;
Terminal=false
EOF
  log "Created launcher at $DESKTOP"

  case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
      warn "$BIN_DIR is not in your PATH. Add it, e.g.:"
      warn "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> \"\$HOME/.profile\""
      ;;
  esac

  log "Done. Run with: $BIN_NAME"
}

install_macos() {
  MOUNT_DIR="$TMP/mount"
  mkdir -p "$MOUNT_DIR"
  log "Mounting dmg…"
  hdiutil attach "$TMP/payload" -nobrowse -quiet -mountpoint "$MOUNT_DIR"

  APP_SRC="$(ls -d "$MOUNT_DIR"/*.app 2>/dev/null | head -1)"
  [ -n "$APP_SRC" ] || {
    hdiutil detach "$MOUNT_DIR" -quiet || true
    die "no .app found in the dmg"
  }

  APP_NAME="$(basename "$APP_SRC")"
  APP_DEST="/Applications/$APP_NAME"

  if [ -w /Applications ]; then
    COPY="cp -R"
  else
    warn "/Applications is not user-writable — using sudo"
    COPY="sudo cp -R"
  fi
  rm -rf "$APP_DEST" 2>/dev/null || sudo rm -rf "$APP_DEST"
  $COPY "$APP_SRC" "$APP_DEST"
  hdiutil detach "$MOUNT_DIR" -quiet
  log "Installed $APP_DEST"
  log "Done. Launch from Spotlight or: open -a Ldapex"
}

case "$OS_KEY" in
  linux) install_linux ;;
  macos) install_macos ;;
esac
