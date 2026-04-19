use std::{collections::HashSet, time::Duration};

use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry, SearchOptions};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, instrument};

use crate::{
    error::{LdapexError, Result},
    schema::{attribute_name, parse_object_class},
    types::{Attribute, AttributeValue, DnLabel, Entry, Modification, SchemaInfo, SearchParams},
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
        check_rc(res.rc, res.text)
    }

    /// Return the direct children of `base_dn` (scope `onelevel`).
    /// Entries come back with `objectClass` populated so the UI can
    /// pick an icon without a second round trip.
    #[instrument(skip(self), fields(base = %base_dn))]
    pub async fn list_children(&self, base_dn: &str) -> Result<Vec<DnLabel>> {
        let attrs = ["objectClass", "cn", "ou", "dc", "uid"];
        let entries = self
            .raw_search(base_dn, Scope::OneLevel, "(objectClass=*)", &attrs, None)
            .await?;
        Ok(entries.iter().map(DnLabel::from_entry).collect())
    }

    /// Read a single entry (scope `base`). Operational attributes are
    /// not requested here; Phase 3 can add an optional flag for them.
    #[instrument(skip(self), fields(dn = %dn))]
    pub async fn read_entry(&self, dn: &str) -> Result<Entry> {
        let entries = self
            .raw_search(dn, Scope::Base, "(objectClass=*)", &["*"], None)
            .await?;
        entries
            .into_iter()
            .next()
            .ok_or_else(|| LdapexError::NoSuchObject(dn.to_string()))
    }

    /// Arbitrary LDAP search exposed to the UI (Phase 2).
    #[instrument(skip(self, params), fields(base = %params.base_dn, scope = ?params.scope))]
    pub async fn search(&self, params: SearchParams) -> Result<Vec<Entry>> {
        let filter = if params.filter.trim().is_empty() {
            "(objectClass=*)".to_string()
        } else {
            params.filter.clone()
        };
        let attr_refs: Vec<&str> = if params.attributes.is_empty() {
            vec!["*"]
        } else {
            params.attributes.iter().map(String::as_str).collect()
        };
        self.raw_search(
            &params.base_dn,
            params.scope.into(),
            &filter,
            &attr_refs,
            params.size_limit,
        )
        .await
    }

    /// Apply a batch of attribute-level modifications (RFC 4511 §4.6).
    #[instrument(skip(self, mods), fields(dn = %dn, n_mods = mods.len()))]
    pub async fn modify(&self, dn: &str, mods: &[Modification]) -> Result<()> {
        let ldap_mods: Vec<ldap3::Mod<String>> = mods.iter().map(to_ldap_mod).collect();
        let mut guard = self.inner.lock().await;
        let res = guard.modify(dn, ldap_mods).await?;
        check_rc(res.rc, res.text)
    }

    /// Create a new entry. Only text values are sent; binary creation
    /// can come in a later revision (Phase 2+ of the write path).
    #[instrument(skip(self, attributes), fields(dn = %dn, n_attrs = attributes.len()))]
    pub async fn add(&self, dn: &str, attributes: &[Attribute]) -> Result<()> {
        let payload: Vec<(String, HashSet<String>)> = attributes
            .iter()
            .map(|a| {
                let values: HashSet<String> = a
                    .values
                    .iter()
                    .filter_map(|v| match v {
                        AttributeValue::Text(t) => Some(t.clone()),
                        AttributeValue::Binary(_) => None,
                    })
                    .collect();
                (a.name.clone(), values)
            })
            .collect();
        let mut guard = self.inner.lock().await;
        let res = guard.add(dn, payload).await?;
        check_rc(res.rc, res.text)
    }

    /// Delete a leaf entry. Recursive deletion is intentionally not
    /// offered here — it belongs to a guarded UI path.
    #[instrument(skip(self), fields(dn = %dn))]
    pub async fn delete(&self, dn: &str) -> Result<()> {
        let mut guard = self.inner.lock().await;
        let res = guard.delete(dn).await?;
        check_rc(res.rc, res.text)
    }

    /// Rename or move an entry (`modifydn`). `new_parent = None` keeps
    /// the entry under the same parent.
    #[instrument(skip(self), fields(dn = %dn, new_rdn = %new_rdn))]
    pub async fn rename(
        &self,
        dn: &str,
        new_rdn: &str,
        new_parent: Option<&str>,
        delete_old_rdn: bool,
    ) -> Result<()> {
        let mut guard = self.inner.lock().await;
        let res = guard
            .modifydn(dn, new_rdn, delete_old_rdn, new_parent)
            .await?;
        check_rc(res.rc, res.text)
    }

    /// Fetch a usable subset of the server schema.
    ///
    /// The function first reads the root DSE to locate the subschema
    /// subentry, then parses `attributeTypes` and `objectClasses` from
    /// it. Servers that decline to publish either are handled
    /// gracefully (the corresponding list is empty).
    #[instrument(skip(self))]
    pub async fn fetch_schema(&self) -> Result<SchemaInfo> {
        let root_dse = self
            .raw_search(
                "",
                Scope::Base,
                "(objectClass=*)",
                &["subschemaSubentry"],
                Some(1),
            )
            .await?;
        let subschema_dn = root_dse
            .into_iter()
            .next()
            .and_then(|e| {
                e.attributes
                    .into_iter()
                    .find(|a| a.name.eq_ignore_ascii_case("subschemaSubentry"))
                    .and_then(|a| {
                        a.values.into_iter().find_map(|v| match v {
                            AttributeValue::Text(t) => Some(t),
                            AttributeValue::Binary(_) => None,
                        })
                    })
            })
            .unwrap_or_else(|| "cn=Subschema".to_string());

        let subschema = self
            .raw_search(
                &subschema_dn,
                Scope::Base,
                "(objectClass=subschema)",
                &["attributeTypes", "objectClasses"],
                Some(1),
            )
            .await?;

        let entry = subschema
            .into_iter()
            .next()
            .ok_or_else(|| LdapexError::NoSuchObject(subschema_dn.clone()))?;

        let mut attribute_names: Vec<String> = Vec::new();
        let mut object_classes = Vec::new();

        for attr in entry.attributes {
            match attr.name.to_ascii_lowercase().as_str() {
                "attributetypes" => {
                    attribute_names.extend(attr.values.into_iter().filter_map(|v| match v {
                        AttributeValue::Text(t) => attribute_name(&t),
                        AttributeValue::Binary(_) => None,
                    }));
                }
                "objectclasses" => {
                    object_classes.extend(attr.values.into_iter().filter_map(|v| match v {
                        AttributeValue::Text(t) => parse_object_class(&t),
                        AttributeValue::Binary(_) => None,
                    }));
                }
                _ => {}
            }
        }

        attribute_names.sort();
        attribute_names.dedup();
        object_classes.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });

        Ok(SchemaInfo {
            subschema_dn,
            attribute_names,
            object_classes,
        })
    }

    /// Tear down the session gracefully (sends an LDAP unbind).
    pub async fn disconnect(self) -> Result<()> {
        let mut ldap = self.inner.into_inner();
        ldap.unbind().await?;
        Ok(())
    }

    /// Low-level search used by all search-flavoured methods.
    async fn raw_search(
        &self,
        base: &str,
        scope: Scope,
        filter: &str,
        attrs: &[&str],
        size_limit: Option<u32>,
    ) -> Result<Vec<Entry>> {
        let mut guard = self.inner.lock().await;
        if let Some(limit) = size_limit {
            let opts = SearchOptions::new().sizelimit(i32::try_from(limit).unwrap_or(i32::MAX));
            guard.with_search_options(opts);
        }
        let (raw_entries, res) = guard.search(base, scope, filter, attrs).await?.success()?;
        drop(res);

        Ok(raw_entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(entry_from_search)
            .collect())
    }
}

fn check_rc(rc: u32, text: String) -> Result<()> {
    if rc == 0 {
        Ok(())
    } else {
        Err(LdapexError::from_result_code(rc, text))
    }
}

fn to_ldap_mod(m: &Modification) -> ldap3::Mod<String> {
    match m {
        Modification::Add { attribute, values } => {
            ldap3::Mod::Add(attribute.clone(), values.iter().cloned().collect())
        }
        Modification::Replace { attribute, values } => {
            ldap3::Mod::Replace(attribute.clone(), values.iter().cloned().collect())
        }
        Modification::Delete { attribute, values } => {
            let set: HashSet<String> = values.clone().unwrap_or_default().into_iter().collect();
            ldap3::Mod::Delete(attribute.clone(), set)
        }
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
        // Merge into an existing attribute when ldap3 splits the same
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

    #[test]
    fn modification_serde_round_trip_delete_all() {
        let m = Modification::Delete {
            attribute: "description".into(),
            values: None,
        };
        let s = serde_json::to_string(&m).unwrap();
        let back: Modification = serde_json::from_str(&s).unwrap();
        assert_eq!(m, back);
    }
}
