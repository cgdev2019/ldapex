use std::time::Duration;

use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, instrument};

use crate::{
    error::{LdapexError, Result},
    types::{Attribute, AttributeValue, DnLabel, Entry},
};

/// TLS negotiation strategy. `None` is only valid over loopback or a
/// trusted segment; the UI warns explicitly before letting the user pick
/// it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TlsMode {
    #[default]
    None,
    StartTls,
    Ldaps,
}

/// Connection parameters fed to [`LdapClient::connect`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectOptions {
    /// LDAP URL, e.g. `ldap://dc.example.org:389` or `ldaps://…`.
    pub url: String,
    #[serde(default)]
    pub tls: TlsMode,
    /// Per-operation timeout. A missing value uses ldap3's default
    /// (no timeout); we set 30 s as a sane ceiling for the UI.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl ConnectOptions {
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            tls: TlsMode::default(),
            timeout_secs: Some(30),
        }
    }
}

/// Thread-safe async LDAP client. The backing [`ldap3::Ldap`] handle is
/// guarded by a `tokio::sync::Mutex` so multiple Tauri commands can
/// share the same session.
pub struct LdapClient {
    inner: Mutex<ldap3::Ldap>,
    options: ConnectOptions,
}

impl LdapClient {
    /// Open a connection to the directory. No bind is performed — call
    /// [`LdapClient::simple_bind`] next.
    #[instrument(skip_all, fields(url = %options.url, tls = ?options.tls))]
    pub async fn connect(options: ConnectOptions) -> Result<Self> {
        let settings = match options.tls {
            TlsMode::StartTls => LdapConnSettings::new().set_starttls(true),
            // LDAPS is driven by the URL scheme (`ldaps://`), not a flag.
            TlsMode::None | TlsMode::Ldaps => LdapConnSettings::new(),
        };

        let (conn, ldap) = LdapConnAsync::with_settings(settings, &options.url).await?;
        ldap3::drive!(conn);

        let mut client = Self {
            inner: Mutex::new(ldap),
            options,
        };

        if let Some(secs) = client.options.timeout_secs {
            client
                .inner
                .get_mut()
                .with_timeout(Duration::from_secs(secs));
        }

        debug!("ldap connection open");
        Ok(client)
    }

    /// Bind with a DN + password. An empty `bind_dn` performs an
    /// anonymous bind.
    #[instrument(skip(self, password), fields(dn = %bind_dn))]
    pub async fn simple_bind(&self, bind_dn: &str, password: &str) -> Result<()> {
        let mut guard = self.inner.lock().await;
        let res = guard.simple_bind(bind_dn, password).await?;
        if res.rc != 0 {
            return Err(LdapexError::from_result_code(res.rc, res.text));
        }
        Ok(())
    }

    /// Return the direct children of `base_dn` (scope `onelevel`).
    /// The entries come back with `objectClass` populated so the UI can
    /// pick an icon without a second round trip.
    #[instrument(skip(self), fields(base = %base_dn))]
    pub async fn list_children(&self, base_dn: &str) -> Result<Vec<DnLabel>> {
        let attrs = ["objectClass", "cn", "ou", "dc", "uid"];
        let entries = self
            .search(base_dn, Scope::OneLevel, "(objectClass=*)", &attrs)
            .await?;
        Ok(entries.iter().map(DnLabel::from_entry).collect())
    }

    /// Read a single entry (scope `base`). Operational attributes are
    /// not requested here; Phase 2 will add an optional flag for them.
    #[instrument(skip(self), fields(dn = %dn))]
    pub async fn read_entry(&self, dn: &str) -> Result<Entry> {
        let entries = self
            .search(dn, Scope::Base, "(objectClass=*)", &["*"])
            .await?;
        entries
            .into_iter()
            .next()
            .ok_or_else(|| LdapexError::NoSuchObject(dn.to_string()))
    }

    /// Low-level search used by [`list_children`] and [`read_entry`].
    async fn search(
        &self,
        base: &str,
        scope: Scope,
        filter: &str,
        attrs: &[&str],
    ) -> Result<Vec<Entry>> {
        let mut guard = self.inner.lock().await;
        let (raw_entries, res) = guard.search(base, scope, filter, attrs).await?.success()?;
        drop(res);

        Ok(raw_entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(entry_from_search)
            .collect())
    }

    /// Tear down the session gracefully (sends an LDAP unbind).
    pub async fn disconnect(self) -> Result<()> {
        let mut ldap = self.inner.into_inner();
        ldap.unbind().await?;
        Ok(())
    }
}

/// Convert an ldap3 [`SearchEntry`] into our domain [`Entry`].
/// Binary attributes land in the same `Attribute` as their text siblings;
/// the frontend branches on the `AttributeValue::{Text,Binary}` tag.
fn entry_from_search(se: SearchEntry) -> Entry {
    let mut attributes: Vec<Attribute> = se
        .attrs
        .into_iter()
        .map(|(name, values)| Attribute {
            name,
            values: values.into_iter().map(AttributeValue::Text).collect(),
        })
        .collect();

    for (name, values) in se.bin_attrs {
        let bin_values: Vec<AttributeValue> = values
            .iter()
            .map(|bytes| AttributeValue::binary(bytes))
            .collect();
        // Merge into an existing attribute when ldap3 split the same
        // name between `attrs` and `bin_attrs` (rare but happens for
        // dual-syntax attributes like userSMIMECertificate).
        if let Some(existing) = attributes
            .iter_mut()
            .find(|a| a.name.eq_ignore_ascii_case(&name))
        {
            existing.values.extend(bin_values);
        } else {
            attributes.push(Attribute {
                name,
                values: bin_values,
            });
        }
    }

    attributes.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    Entry {
        dn: se.dn,
        attributes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldap3::SearchEntry;
    use std::collections::HashMap;

    #[test]
    fn entry_from_search_merges_bin_and_text() {
        let mut attrs = HashMap::new();
        attrs.insert("cn".to_string(), vec!["Alice".to_string()]);
        attrs.insert("objectClass".to_string(), vec!["inetOrgPerson".to_string()]);

        let mut bin_attrs = HashMap::new();
        bin_attrs.insert("userCertificate;binary".to_string(), vec![vec![0xDE, 0xAD]]);

        let se = SearchEntry {
            dn: "cn=Alice,dc=ex".into(),
            attrs,
            bin_attrs,
        };

        let entry = entry_from_search(se);
        assert_eq!(entry.dn, "cn=Alice,dc=ex");
        // Alphabetical sort: cn < objectClass < userCertificate;binary
        assert_eq!(entry.attributes[0].name, "cn");
        assert_eq!(entry.attributes[2].name, "userCertificate;binary");
        assert!(matches!(
            entry.attributes[2].values[0],
            AttributeValue::Binary(ref s) if s == "3q0="
        ));
    }
}
