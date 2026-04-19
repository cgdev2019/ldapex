# ADR-0001 — Native shell: Tauri v2

- **Status**: accepted
- **Date**: 2026-04-19
- **Deciders**: Ldapex team

## Context

Ldapex is a desktop application (not a web app). Its backend is written
in Rust and its UI in Svelte. We need a "native shell": the component
that hosts the web UI in an OS window, exposes a typed IPC bridge to
the Rust code, and can build native installers (`.deb`/`.AppImage`,
`.dmg`, `.msi`).

Functional requirements that drive the choice:

- Run async Rust (tokio) to talk to an LDAP directory through `ldap3`.
- Access the **OS keyring** (Secret Service on Linux, Keychain on
  macOS, Credential Manager on Windows).
- Negotiate **TLS / StartTLS** from Rust, with support for a custom
  trust anchor.
- Produce **lightweight** binaries (no bundled Chromium) on all three
  target OSes.
- Stay in a **Rust-first** ecosystem (no hard dependency on a Node
  runtime for the app itself).

## Alternatives considered

### Tauri v2 — selected

- Reference Rust wrapper: it combines `wry` (webview) and `tao`
  (windowing).
- Uses the **system webview** (WebView2 on Windows, WKWebView on
  macOS, WebKitGTK on Linux). Final binaries weigh ~5–15 MB.
- Native bundling out of the box (`cargo tauri build` → `.deb`,
  `.AppImage`, `.dmg`, `.msi`).
- Typed IPC with `#[tauri::command]` plus official plugins `keyring`,
  `fs`, `shell`, `dialog`, `updater`.
- Works with any static frontend — SvelteKit in `adapter-static` mode
  fits.
- v2 is stable, the ecosystem is active, and macOS/Windows signing are
  first-class in CI.

### Wry + Tao, standalone — rejected

- The low-level building blocks behind Tauri. Ultimate control, but
  forces us to reimplement IPC, the bundler, the permissions model,
  and every system plugin.
- **No upside** over Tauri for our needs. To be revisited only if
  Tauri becomes a blocker.

### Dioxus Desktop — rejected

- Full-Rust stack with Rust RSX-style components.
- Requires dropping **Svelte**, which is explicitly listed as our
  frontend stack. That is a structural change, out of scope.

### Electron / Neutralino — rejected

- Electron: bundles Chromium (~100–150 MB), no tokio on the main
  process, integrating Rust means a side-process and fragile IPC.
- Neutralino: no first-class Rust backend.
- Both violate the "Rust-first + lightweight binaries" constraint.

## Decision

We use **Tauri v2** as the native shell.

## Consequences

**Positive**

- Direct integration with `ldap3`, `keyring`, `rustls` in the same
  Rust process.
- Multi-OS packaging and code signing are covered by Tauri's tooling.
- Svelte stays free on the frontend side, bridged via
  `@tauri-apps/api`.
- Granular permissions via the Tauri v2 capabilities model.

**Negative / watch out for**

- Rendering differences between WebView2, WKWebView and WebKitGTK:
  we must keep systematic tests on all three OSes (Phase 4, E2E task).
- WebKitGTK historically lags behind on CSS/JS features; we avoid
  bleeding-edge web APIs while we target Linux.
- Strong dependency on the Tauri ecosystem — any major upstream change
  (v3, plugin migrations) requires revisiting this ADR.

## Follow-up

- Code structure: `ldapex-core` crate (pure Rust, independently
  testable) + `ldapex-app` crate (Tauri binary that consumes
  `ldapex-core` and exposes the commands).
- Any change of native shell requires a new ADR that supersedes this
  one.
