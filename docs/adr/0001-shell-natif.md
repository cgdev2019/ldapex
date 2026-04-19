# ADR-0001 — Shell natif : Tauri v2

- **Statut** : accepté
- **Date** : 2026-04-19
- **Décideurs** : équipe Ldapex

## Contexte

Ldapex est une application de bureau (non-web) dont le backend est écrit en
Rust et l'interface en Svelte. Il nous faut un « shell natif » : le composant
qui héberge l'interface web dans une fenêtre système, expose un pont IPC
typé vers le code Rust, et sait produire des paquets natifs
(`.deb`/`.AppImage`, `.dmg`, `.msi`).

Besoins fonctionnels qui pèsent dans le choix :

- Exécuter du Rust asynchrone (tokio) pour parler à un annuaire LDAP via
  `ldap3`.
- Accéder au **trousseau de clefs de l'OS** (Secret Service sur Linux,
  Keychain sur macOS, Credential Manager sur Windows).
- Négocier **TLS / StartTLS** côté Rust, avec support d'une CA custom.
- Produire des binaires **légers** (pas de Chromium embarqué) sur les
  trois OS cibles.
- Rester dans un écosystème **Rust-first** (pas de dépendance forte à
  Node au runtime).

## Alternatives envisagées

### Tauri v2 — retenu

- Wrapper Rust de référence : combine `wry` (webview) et `tao` (fenêtre).
- Utilise la webview **système** (WebView2 sur Windows, WKWebView sur
  macOS, WebKitGTK sur Linux) : binaires ~5–15 Mo.
- Bundling natif de série (`cargo tauri build` → `.deb`, `.AppImage`,
  `.dmg`, `.msi`).
- IPC typée `#[tauri::command]` + plugins officiels `keyring`, `fs`,
  `shell`, `dialog`, `updater`.
- Compatible avec n'importe quel frontend statique (SvelteKit en mode
  `adapter-static` convient).
- v2 stabilisée, écosystème actif, signing macOS/Windows intégré au CI.

### Wry + Tao nus — écarté

- Briques bas niveau qui composent Tauri. Donne un contrôle total mais
  oblige à réimplémenter l'IPC, le bundler, le modèle de permissions et
  les plugins système.
- **Aucun bénéfice** par rapport à Tauri pour nos besoins. À reconsidérer
  uniquement si Tauri devient bloquant.

### Dioxus Desktop — écarté

- Stack Rust end-to-end avec composants Rust type RSX.
- Impose d'abandonner **Svelte**, qui est explicitement au README comme
  techno frontend. Décision structurante, hors scope.

### Electron / Neutralino — écarté

- Electron : embarque Chromium (~100–150 Mo), pas de tokio côté main
  process, aligner Rust implique un process séparé et de l'IPC fragile.
- Neutralino : pas de backend Rust de première classe.
- Les deux violent le critère « Rust-first + binaires légers ».

## Décision

Nous utilisons **Tauri v2** comme shell natif.

## Conséquences

**Positives**

- Intégration directe avec `ldap3`, `keyring`, `rustls` dans le même
  process Rust.
- Packaging multi-OS et signing couverts par l'outillage Tauri.
- Svelte reste libre côté frontend, couplé via `@tauri-apps/api`.
- Permissions granulaires via le modèle de capabilities Tauri v2.

**Négatives / à surveiller**

- Différences de rendu entre WebView2, WKWebView et WebKitGTK : à tester
  systématiquement sur les trois OS (Phase 4, tâche E2E).
- WebKitGTK a historiquement des retards de fonctionnalités CSS/JS ; on
  évite les API web trop récentes tant qu'on vise Linux.
- Dépendance forte à l'écosystème Tauri ; tout changement majeur (v3,
  migration de plugins) impliquera une revue de cet ADR.

## Suivi

- Structure du code : crate `ldapex-core` (pur Rust, testable
  indépendamment) + crate `ldapex-app` (binaire Tauri qui consomme
  `ldapex-core` et expose les commandes).
- Tout changement de shell natif exige un nouvel ADR qui supersède
  celui-ci.
