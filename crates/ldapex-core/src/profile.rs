//! Connection profiles — user-saved LDAP endpoints.
//!
//! Profiles are pure data: the storage layer (TOML file) lives in the
//! `ldapex-app` crate because it uses OS-specific paths.
//!
//! **Security note.** Profiles include the bind password directly in
//! the file. The storage layer writes the file with 0600 permissions
//! on Unix. See README §"Profiles and secrets" for the trade-off.

use serde::{Deserialize, Serialize};

use crate::client::TlsMode;

/// Monotonic version of the profile file format. Increment when
/// changing fields to support migrations.
pub const PROFILE_SCHEMA_VERSION: u32 = 2;

/// One saved LDAP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionProfile {
    /// Stable identifier (UUID v4 or any unique string).
    pub id: String,

    /// Human-readable label shown in the profile picker.
    pub name: String,

    /// LDAP URL, e.g. `ldaps://dc.example.org:636`.
    pub url: String,

    /// Distinguished Name used for the bind. Empty = anonymous bind.
    #[serde(default)]
    pub bind_dn: String,

    /// Suffix to display in the tree after a successful bind.
    pub base_dn: String,

    #[serde(default)]
    pub tls: TlsMode,

    /// Optional per-operation timeout. Falls back to the client
    /// default (30 s) if missing.
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Bind password persisted in the profile file. `None` (or missing
    /// in TOML) means the UI prompts for it at every connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl ConnectionProfile {
    /// Build a new profile with no password and default TLS.
    #[must_use]
    pub fn new_with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url: url.into(),
            bind_dn: String::new(),
            base_dn: String::new(),
            tls: TlsMode::default(),
            timeout_secs: Some(30),
            password: None,
        }
    }
}

/// Root of the persisted profile file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStore {
    /// Format version — migration hook.
    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default)]
    pub profiles: Vec<ConnectionProfile>,
}

fn default_version() -> u32 {
    PROFILE_SCHEMA_VERSION
}

impl ProfileStore {
    /// Replace or insert a profile based on `id`.
    pub fn upsert(&mut self, profile: ConnectionProfile) {
        if let Some(existing) = self.profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    /// Remove by id. Returns the removed profile, if any.
    pub fn remove(&mut self, id: &str) -> Option<ConnectionProfile> {
        let pos = self.profiles.iter().position(|p| p.id == id)?;
        Some(self.profiles.remove(pos))
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ConnectionProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_inserts_then_updates() {
        let mut store = ProfileStore::default();
        let profile = ConnectionProfile::new_with_id("a", "A", "ldap://a");
        store.upsert(profile.clone());
        assert_eq!(store.profiles.len(), 1);

        let mut updated = profile.clone();
        updated.name = "A renamed".into();
        store.upsert(updated);
        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.profiles[0].name, "A renamed");
    }

    #[test]
    fn remove_returns_profile() {
        let mut store = ProfileStore::default();
        store.upsert(ConnectionProfile::new_with_id("a", "A", "ldap://a"));
        store.upsert(ConnectionProfile::new_with_id("b", "B", "ldap://b"));
        let removed = store.remove("a").expect("a present");
        assert_eq!(removed.id, "a");
        assert_eq!(store.profiles.len(), 1);
        assert!(store.remove("a").is_none());
    }

    #[test]
    fn serde_round_trip_toml() {
        let mut store = ProfileStore {
            version: PROFILE_SCHEMA_VERSION,
            profiles: Vec::new(),
        };
        store.upsert(ConnectionProfile {
            id: "1".into(),
            name: "Prod".into(),
            url: "ldaps://ldap.corp:636".into(),
            bind_dn: "cn=svc,dc=corp".into(),
            base_dn: "dc=corp".into(),
            tls: TlsMode::Ldaps,
            timeout_secs: Some(15),
            password: Some("s3cret".into()),
        });

        let text = toml::to_string(&store).expect("serialize");
        let back: ProfileStore = toml::from_str(&text).expect("deserialize");
        assert_eq!(store.profiles, back.profiles);
        assert_eq!(store.version, back.version);
    }

    #[test]
    fn password_field_is_omitted_when_none() {
        let store = ProfileStore {
            version: PROFILE_SCHEMA_VERSION,
            profiles: vec![ConnectionProfile::new_with_id("a", "A", "ldap://a")],
        };
        let text = toml::to_string(&store).expect("serialize");
        assert!(
            !text.contains("password"),
            "password should be omitted when None: {text}",
        );
    }
}
