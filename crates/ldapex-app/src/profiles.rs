//! Profile persistence: TOML file on disk + OS keyring for passwords.
//!
//! The on-disk file lives next to the rest of the app's config so the
//! user can back it up / diff it with a text tool. Passwords are
//! **never** written to it — they are held in the OS keyring under
//! service name `LDAPEX_SERVICE` with the profile id as account.

use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::Mutex,
};

use ldapex_core::{ConnectionProfile, ProfileStore, PROFILE_SCHEMA_VERSION};
use thiserror::Error;

/// Service identifier for the OS keyring. Keep stable — renaming it
/// orphans every stored password.
pub const LDAPEX_SERVICE: &str = "ldapex";

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("config directory unavailable on this platform")]
    NoConfigDir,
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    #[error("TOML parse: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("TOML serialize: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("keyring: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("profile not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ProfileError>;

/// Owns the on-disk profile store behind a lock. All public methods
/// reload from disk on read and rewrite on write so concurrent CLI
/// edits stay consistent.
pub struct ProfileManager {
    path: PathBuf,
    lock: Mutex<()>,
}

impl ProfileManager {
    /// Initialise the manager, creating the config directory if needed.
    pub fn new() -> Result<Self> {
        let path = config_file()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self {
            path,
            lock: Mutex::new(()),
        })
    }

    #[cfg(test)]
    fn with_path(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }

    /// Load the full profile list from disk (empty store on first use).
    pub fn list(&self) -> Result<Vec<ConnectionProfile>> {
        let _g = self.lock.lock().unwrap();
        Ok(self.load_unlocked()?.profiles)
    }

    /// Insert or replace a profile.
    pub fn save(&self, profile: ConnectionProfile) -> Result<ConnectionProfile> {
        let _g = self.lock.lock().unwrap();
        let mut store = self.load_unlocked()?;
        store.upsert(profile.clone());
        self.persist_unlocked(&store)?;
        Ok(profile)
    }

    /// Remove a profile **and** its stored password if any.
    pub fn delete(&self, id: &str) -> Result<()> {
        let _g = self.lock.lock().unwrap();
        let mut store = self.load_unlocked()?;
        if store.remove(id).is_none() {
            return Err(ProfileError::NotFound(id.into()));
        }
        self.persist_unlocked(&store)?;
        // Best-effort keyring cleanup — a missing entry is fine.
        if let Ok(entry) = keyring::Entry::new(LDAPEX_SERVICE, id) {
            let _ = entry.delete_credential();
        }
        Ok(())
    }

    /// Export every profile to a JSON document (used by "Export…").
    pub fn export_json(&self) -> Result<String> {
        let store = {
            let _g = self.lock.lock().unwrap();
            self.load_unlocked()?
        };
        Ok(serde_json::to_string_pretty(&store)?)
    }

    /// Import a JSON document (replacing any same-id profiles, keeping
    /// the others). Returns the merged list.
    pub fn import_json(&self, json: &str) -> Result<Vec<ConnectionProfile>> {
        let incoming: ProfileStore = serde_json::from_str(json)?;
        let _g = self.lock.lock().unwrap();
        let mut store = self.load_unlocked()?;
        for p in incoming.profiles {
            store.upsert(p);
        }
        self.persist_unlocked(&store)?;
        Ok(store.profiles)
    }

    /// Retrieve the saved password for a profile, if any.
    pub fn get_password(&self, id: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(LDAPEX_SERVICE, id)?;
        match entry.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Write (or replace) the password for a profile in the OS keyring.
    pub fn set_password(&self, id: &str, password: &str) -> Result<()> {
        let entry = keyring::Entry::new(LDAPEX_SERVICE, id)?;
        entry.set_password(password)?;
        Ok(())
    }

    /// Remove a stored password (no error if absent).
    pub fn clear_password(&self, id: &str) -> Result<()> {
        let entry = keyring::Entry::new(LDAPEX_SERVICE, id)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn load_unlocked(&self) -> Result<ProfileStore> {
        match fs::read_to_string(&self.path) {
            Ok(text) => {
                let mut store: ProfileStore = toml::from_str(&text)?;
                store.version = PROFILE_SCHEMA_VERSION;
                Ok(store)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(ProfileStore::default()),
            Err(e) => Err(e.into()),
        }
    }

    fn persist_unlocked(&self, store: &ProfileStore) -> Result<()> {
        let text = toml::to_string_pretty(store)?;
        write_atomic(&self.path, text.as_bytes())
    }
}

/// Write `content` to `path` via a sibling temp file + rename so a
/// crash mid-write leaves the previous file intact.
fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "profile path has no parent directory"))?;
    let tmp = dir.join(format!(".{}.tmp", random_suffix()));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn random_suffix() -> String {
    uuid::Uuid::new_v4().as_simple().to_string()
}

/// Canonical config file path: `<config>/ldapex/profiles.toml`.
pub fn config_file() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or(ProfileError::NoConfigDir)?;
    Ok(base.join("ldapex").join("profiles.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldapex_core::ConnectionProfile;
    use tempfile::tempdir;

    #[test]
    fn save_and_list_round_trip() {
        let dir = tempdir().expect("tempdir");
        let mgr = ProfileManager::with_path(dir.path().join("profiles.toml"));
        mgr.save(ConnectionProfile::new_with_id("a", "A", "ldap://a"))
            .expect("save");
        mgr.save(ConnectionProfile::new_with_id("b", "B", "ldap://b"))
            .expect("save");

        let list = mgr.list().expect("list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "a");
    }

    #[test]
    fn delete_removes_from_file() {
        let dir = tempdir().expect("tempdir");
        let mgr = ProfileManager::with_path(dir.path().join("profiles.toml"));
        mgr.save(ConnectionProfile::new_with_id("a", "A", "ldap://a"))
            .expect("save");
        mgr.delete("a").expect("delete");
        assert!(mgr.list().expect("list").is_empty());
        assert!(matches!(
            mgr.delete("a").unwrap_err(),
            ProfileError::NotFound(_)
        ));
    }

    #[test]
    fn import_merges_with_existing() {
        let dir = tempdir().expect("tempdir");
        let mgr = ProfileManager::with_path(dir.path().join("profiles.toml"));
        mgr.save(ConnectionProfile::new_with_id("a", "A", "ldap://a"))
            .expect("save");

        let json = r#"{
            "version": 1,
            "profiles": [
                { "id": "b", "name": "B", "url": "ldap://b", "bind_dn": "", "base_dn": "", "tls": "none", "save_password": false }
            ]
        }"#;
        let merged = mgr.import_json(json).expect("import");
        assert_eq!(merged.len(), 2);
    }
}
