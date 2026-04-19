# Installation

Ldapex ships a single distributable file per operating system. Grab the
one matching your platform from the project's GitHub releases page and
you are ready to go.

## Supported platforms

| OS      | Installer                     | Runtime requirement                 |
| ------- | ----------------------------- | ----------------------------------- |
| Windows | `ldapex_*_setup.exe` (NSIS)   | None — WebView2 bootstrapper is embedded. |
| macOS   | `Ldapex_*.dmg`                | macOS 11 or newer.                  |
| Linux   | `ldapex_*.AppImage`           | `libwebkit2gtk-4.1` available on the host. |

Bare binaries at `target/release/ldapex-app(.exe)` are also self-
contained — the Svelte frontend is compiled into the Rust executable.

## Windows

1. Download `ldapex_<version>_setup.exe`.
2. Double-click and follow the installer.
3. Launch **Ldapex** from the Start menu.

## macOS

1. Download `Ldapex_<version>.dmg`.
2. Open the disk image and drag the Ldapex app into Applications.
3. The first launch requires right-click → *Open* because the bundle
   is not yet notarised.

(todo: add screenshot of the macOS Gatekeeper dialog)

## Linux (AppImage)

```bash
chmod +x ldapex_<version>.AppImage
./ldapex_<version>.AppImage
```

The AppImage needs `libwebkit2gtk-4.1` at runtime. On Debian/Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1
```

## Building from source

If you prefer to build the app yourself:

```bash
git clone https://github.com/cgdev2019/ldapex.git
cd ldapex
npm --prefix frontend install
cargo install tauri-cli --version '^2.0' --locked
cargo tauri build
```

The resulting installers land in `target/release/bundle/`.
