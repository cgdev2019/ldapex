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

```bash
cargo tauri build
# → artefacts dans crates/ldapex-app/target/release/bundle/
```

## Licence

[MIT](LICENSE).
