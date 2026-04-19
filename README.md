# Ldapex

Application de bureau pour visualiser et modifier un annuaire LDAP.

## Stack

- **Backend** : Rust
- **Frontend** : Svelte
- **Mode** : application native (non-web)

## Fonctionnalités

- Connexion à un serveur LDAP (simple bind, éventuellement SASL)
- Navigation de l'arborescence DIT
- Affichage des attributs d'une entrée
- Création, modification, suppression d'entrées
- Recherche par filtre LDAP
- Gestion de plusieurs profils de connexion

## Architecture

- **Shell natif** : Tauri v2 (voir [ADR-0001](docs/adr/0001-shell-natif.md))
- **Backend** : crate `ldapex-core` (logique LDAP, pur Rust, testable) +
  crate `ldapex-app` (binaire Tauri qui expose les commandes au frontend)
- **Frontend** : SvelteKit en mode `adapter-static` (SPA bundlée dans l'app)

```text
ldapex/
├── crates/
│   ├── ldapex-core/   # logique LDAP (ldap3, schéma, erreurs)
│   └── ldapex-app/    # shell Tauri, commandes IPC
├── frontend/          # SvelteKit SPA
└── docs/adr/          # décisions d'architecture
```

## Prérequis

- Rust stable (toolchain pinnée via `rust-toolchain.toml`)
- Node.js ≥ 22
- Tauri CLI : `cargo install tauri-cli --version '^2.0'`
- Un serveur LDAP accessible (OpenLDAP, Active Directory, 389 DS…)
- **Linux uniquement** : paquets WebView2Gtk :
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
      librsvg2-dev libsoup-3.0-dev build-essential pkg-config
  ```

## Installation

```bash
git clone <repo> ldapex
cd ldapex
npm --prefix frontend install
```

## Développement

```bash
# Lance le backend + le dev server Vite + la fenêtre Tauri
cargo tauri dev
```

Scripts utiles :

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

npm --prefix frontend run check   # svelte-check
npm --prefix frontend run lint
```

Hooks git :

```bash
pip install pre-commit
pre-commit install
```

## Build

Tauri v2 embarque le frontend Svelte directement dans le binaire Rust, donc
chaque OS produit **un seul fichier** côté distribution.

```bash
cargo tauri build
```

Artefacts produits dans `target/release/` :

| OS      | Fichier unique                        | Runtime requis                   |
| ------- | ------------------------------------- | -------------------------------- |
| Windows | `bundle/nsis/ldapex_*_setup.exe`      | aucune DLL (CRT statique, WebView2 bootstrapper embarqué) |
| macOS   | `bundle/dmg/Ldapex_*.dmg`             | macOS 11+ (WebKit fourni par l'OS) |
| Linux   | `bundle/appimage/ldapex_*.AppImage`   | `libwebkit2gtk-4.1` sur la machine cible¹ |

Le binaire brut `target/release/ldapex-app(.exe)` reste disponible et est
portable tel quel : frontend embarqué, pas de dossier `resources/`.

¹ WebKitGTK ne peut pas être lié statiquement en pratique. Pour un
AppImage totalement autonome, il faudra bundler WebKit2GTK manuellement
(tâche éventuelle de Phase 4).

## Licence

[MIT](LICENSE).
