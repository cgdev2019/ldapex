# Ldapex

Desktop application to browse and edit an LDAP directory.

## Stack

- **Backend**: Rust
- **Frontend**: Svelte
- **Mode**: native application (not a web app)

## Features

- Connection to an LDAP server (simple bind, SASL planned)
- DIT tree navigation
- Entry attribute display
- Entry create / modify / delete
- LDAP filter search (RFC 4515)
- Multiple saved connection profiles with passwords in the OS keyring

## Architecture

- **Native shell**: Tauri v2 (see [ADR-0001](docs/adr/0001-native-shell.md))
- **Backend**: `ldapex-core` crate (pure-Rust LDAP logic, testable in
  isolation) + `ldapex-app` crate (Tauri binary that exposes IPC
  commands to the frontend)
- **Frontend**: SvelteKit in `adapter-static` mode (SPA bundled into
  the app)

```text
ldapex/
├── crates/
│   ├── ldapex-core/   # LDAP logic (ldap3, schema, errors)
│   └── ldapex-app/    # Tauri shell, IPC commands, profile storage
├── frontend/          # SvelteKit SPA
└── docs/adr/          # architecture decision records
```

## Requirements

- Rust stable (toolchain pinned via `rust-toolchain.toml`)
- Node.js ≥ 22
- Tauri CLI: `cargo install tauri-cli --version '^2.0'`
- An LDAP server to connect to (OpenLDAP, Active Directory, 389 DS, …)
- **Linux only**: WebView2Gtk system packages:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
      librsvg2-dev libsoup-3.0-dev build-essential pkg-config
  ```

## Installation

### End users (pre-built binaries)

Linux / macOS — one-liner that fetches the latest GitHub release and
installs to the standard location (AppImage in `~/.local/bin` on
Linux, `Ldapex.app` in `/Applications` on macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/cgdev2019/ldapex/main/install.sh | sh
```

Windows (PowerShell ≥ 5, elevated):

```powershell
iwr -useb https://raw.githubusercontent.com/cgdev2019/ldapex/main/install.ps1 | iex
```

Pin a specific version with `LDAPEX_VERSION=v1.2.3` (shell) or
`$env:LDAPEX_VERSION='v1.2.3'` (PowerShell).

The installer asks whether to place a Ldapex shortcut on the Desktop.
Skip the prompt with `LDAPEX_DESKTOP_ICON=0` (or `1` to force-create
it non-interactively), or pass `-DesktopIcon No` to the PowerShell
script. Uninstall on Unix with `./uninstall.sh` (use `--all` to also
drop `~/.ldapex`). The uninstaller removes the Desktop shortcut too.

### Developers (from source)

```bash
git clone https://github.com/cgdev2019/ldapex.git
cd ldapex
npm --prefix frontend install
```

## Development

```bash
# Starts the Rust backend, the Vite dev server, and the Tauri window
cargo tauri dev
```

Useful scripts:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

npm --prefix frontend run check   # svelte-check
npm --prefix frontend run lint
```

The app writes daily-rotated logs to `~/.ldapex/logs/ldapex.log.YYYY-MM-DD`
in addition to stdout. Lines that look like `password=…` or `password: …`
are redacted before reaching either sink, so the file is safe to attach
to bug reports. Tune verbosity with `RUST_LOG`, e.g.
`RUST_LOG=ldapex=debug cargo tauri dev`.

Integration tests against a real LDAP server (spins up a bitnami
OpenLDAP container seeded from `docker/openldap/seed.ldif`):

```bash
docker compose -f docker/openldap/docker-compose.yml up -d
cargo test -p ldapex-core --features integration-tests -- --nocapture
docker compose -f docker/openldap/docker-compose.yml down -v
```

Git hooks:

```bash
pip install pre-commit
pre-commit install
```

## Build

Tauri v2 embeds the Svelte frontend directly into the Rust binary, so
each OS produces **a single distributable file**.

```bash
cargo tauri build
```

Artefacts in `target/release/`:

| OS      | Single file                           | Runtime requirement                |
| ------- | ------------------------------------- | ---------------------------------- |
| Windows | `bundle/nsis/ldapex_*_setup.exe`      | none (static CRT, embedded WebView2 bootstrapper) |
| macOS   | `bundle/dmg/Ldapex_*.dmg`             | macOS 11+ (WebKit provided by the OS) |
| Linux   | `bundle/appimage/ldapex_*.AppImage`   | `libwebkit2gtk-4.1` on the target¹ |

The bare binary at `target/release/ldapex-app(.exe)` is also portable
on its own: the frontend is embedded, there is no `resources/` folder
next to it.

¹ WebKitGTK cannot realistically be statically linked. A fully
self-contained AppImage would need to bundle WebKit2GTK manually (a
possible Phase 4 task).

## Profiles and secrets

Saved profiles live in a single TOML file under the user's home
directory, on every OS:

```text
~/.ldapex/profiles.toml
```

The file is created with **0600** permissions on Unix (owner-only
read/write). The parent directory `~/.ldapex/` is created with **0700**.

**Passwords are stored in that file in clear text** when you fill in
the *Password* field of the profile editor. This is a deliberate
trade-off to keep profiles portable and self-contained (just copy the
file to another machine). If you prefer prompt-every-time behaviour,
leave the password blank; the app will ask at each connection.

Export/import operations (clipboard JSON) include passwords too, so be
careful when sharing an exported blob.

## License

[MIT](LICENSE).
