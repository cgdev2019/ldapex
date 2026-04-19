//! Ldapex core — LDAP logic, transport and domain types.
//!
//! The app crate (`ldapex-app`) wraps this crate behind Tauri commands.
//! Nothing here depends on Tauri so that the logic stays testable in
//! isolation.

pub mod error;

pub use error::{LdapexError, Result};

/// Semantic version of the core crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Connectivity smoke-test. Returns a static greeting with the crate
/// version — used by the `ping` Tauri command during bootstrap.
#[must_use]
pub fn ping() -> String {
    format!("ldapex-core {VERSION} ready")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_mentions_version() {
        assert!(ping().contains(VERSION));
    }
}
