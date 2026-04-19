//! Profile persistence: a single TOML file at `~/.ldapex/profiles.toml`.
//!
//! **Security note.** The file contains bind passwords in plain text
//! (the user explicitly opted for a portable, self-contained layout
//! over the OS keyring). We compensate by chmod-ing the file to `0600`
//! on Unix so only the owner can read it. On Windows, NTFS ACLs
//! already restrict the user's profile folder to that user.

use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::Mutex,
};

use ldapex_core::{ConnectionProfile, ProfileStore, PROFILE_SCHEMA_VERSION};
use thiserror::Error;

/// Directory name, relative to the user's home, that hosts every
/// Ldapex config file.
pub const LDAPEX_DIR: &str = ".ldapex";

/// File inside [`LDAPEX_DIR`] that holds the profile list.
pub const PROFILES_FILE: &str = "profiles.toml";

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("home directory unavailable on this platform")]
    NoHomeDir,
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    #[error("TOML parse: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("TOML serialize: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("profile not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ProfileError>;

/// Owns the on-disk profile store behind a lock. All public methods
/// reload from disk on read and rewrite on write so concurrent edits
/// stay consistent.
pub struct ProfileManager {
    path: PathBuf,
    lock: Mutex<()>,
}

impl ProfileManager {
    /// Initialise the manager, creating `~/.ldapex/` if needed.
    pub fn new() -> Result<Self> {
        let path = config_file()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
            restrict_dir_permissions(parent)?;
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

    /// Remove a profile.
    pub fn delete(&self, id: &str) -> Result<()> {
        let _g = self.lock.lock().unwrap();
        let mut store = self.load_unlocked()?;
        if store.remove(id).is_none() {
            return Err(ProfileError::NotFound(id.into()));
        }
        self.persist_unlocked(&store)
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
        write_atomic_restricted(&self.path, text.as_bytes())
    }
}

/// Write `content` to `path` via a sibling temp file + rename so a
/// crash mid-write leaves the previous file intact. On Unix we set
/// `0600` on the temp file **before** renaming, so the final file is
/// never world-readable even for a short window.
fn write_atomic_restricted(path: &Path, content: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "profile path has no parent directory"))?;
    let tmp = dir.join(format!(".{}.tmp", uuid::Uuid::new_v4().as_simple()));
    fs::write(&tmp, content)?;
    restrict_file_permissions(&tmp)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

/// Canonical config file path: `$HOME/.ldapex/profiles.toml`.
pub fn config_file() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(ProfileError::NoHomeDir)?;
    Ok(home.join(LDAPEX_DIR).join(PROFILES_FILE))
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
            "version": 2,
            "profiles": [
                { "id": "b", "name": "B", "url": "ldap://b", "bind_dn": "", "base_dn": "", "tls": "none" }
            ]
        }"#;
        let merged = mgr.import_json(json).expect("import");
        assert_eq!(merged.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn saved_file_is_not_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("profiles.toml");
        let mgr = ProfileManager::with_path(path.clone());
        mgr.save(ConnectionProfile::new_with_id("a", "A", "ldap://a"))
            .expect("save");
        let perms = fs::metadata(&path).expect("stat").permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }
}
