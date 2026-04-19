#!/usr/bin/env sh
# Ldapex uninstaller for Linux and macOS.
#
# Removes files placed by install.sh (binary, launcher shortcut,
# macOS .app bundle). Leaves your data — `~/.ldapex/` — untouched.
# Pass --all to wipe it too.
#
# Overrides:
#   PREFIX=/custom/path     where install.sh placed the Linux binary

set -eu

BIN_NAME="ldapex"
PREFIX="${PREFIX:-$HOME/.local}"
PURGE_DATA=0

for arg in "$@"; do
  case "$arg" in
    --all) PURGE_DATA=1 ;;
    -h|--help)
      cat <<EOF
Usage: uninstall.sh [--all]
  --all    also delete ~/.ldapex (profiles, logs)
EOF
      exit 0
      ;;
    *) ;;
  esac
done

log() { printf '\033[1;34m→\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }

OS="$(uname -s)"

resolve_desktop_dir_linux() {
  if command -v xdg-user-dir >/dev/null 2>&1; then
    d="$(xdg-user-dir DESKTOP 2>/dev/null || true)"
    [ -n "$d" ] && [ -d "$d" ] && { printf '%s' "$d"; return; }
  fi
  [ -d "$HOME/Desktop" ] && { printf '%s' "$HOME/Desktop"; return; }
  [ -d "$HOME/Bureau" ] && { printf '%s' "$HOME/Bureau"; return; }
  printf ''
}

case "$OS" in
  Linux)
    BIN="$PREFIX/bin/$BIN_NAME"
    LAUNCHER="${XDG_DATA_HOME:-$HOME/.local/share}/applications/ldapex.desktop"
    [ -f "$BIN" ] && rm -f "$BIN" && log "removed $BIN" || warn "no binary at $BIN"
    [ -f "$LAUNCHER" ] && rm -f "$LAUNCHER" && log "removed $LAUNCHER" || true
    DESKTOP_DIR="$(resolve_desktop_dir_linux)"
    if [ -n "$DESKTOP_DIR" ] && [ -f "$DESKTOP_DIR/ldapex.desktop" ]; then
      rm -f "$DESKTOP_DIR/ldapex.desktop"
      log "removed $DESKTOP_DIR/ldapex.desktop"
    fi
    ;;
  Darwin)
    APP="/Applications/Ldapex.app"
    if [ -e "$APP" ]; then
      if [ -w /Applications ]; then
        rm -rf "$APP"
      else
        warn "/Applications is not user-writable — using sudo"
        sudo rm -rf "$APP"
      fi
      log "removed $APP"
    else
      warn "no app at $APP"
    fi
    for alias_path in \
      "$HOME/Desktop/Ldapex" \
      "$HOME/Desktop/Ldapex.app" \
      "$HOME/Bureau/Ldapex" \
      "$HOME/Bureau/Ldapex.app"; do
      [ -e "$alias_path" ] && rm -rf "$alias_path" && log "removed $alias_path"
    done
    ;;
  *)
    warn "unsupported OS: $OS (use uninstall.ps1 on Windows)"
    exit 1
    ;;
esac

if [ "$PURGE_DATA" -eq 1 ]; then
  DATA="$HOME/.ldapex"
  [ -d "$DATA" ] && rm -rf "$DATA" && log "removed $DATA"
fi

log "Done."
