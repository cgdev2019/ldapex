use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// Search scope, mirrors the LDAP protocol values of RFC 4511 §4.5.1.2.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    Base,
    OneLevel,
    Subtree,
}

impl From<SearchScope> for ldap3::Scope {
    fn from(s: SearchScope) -> Self {
        match s {
            SearchScope::Base => ldap3::Scope::Base,
            SearchScope::OneLevel => ldap3::Scope::OneLevel,
            SearchScope::Subtree => ldap3::Scope::Subtree,
        }
    }
}

/// Parameters for [`crate::LdapClient::search`]. Empty `attributes`
/// asks the server for all user attributes (`*`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub base_dn: String,
    pub scope: SearchScope,
    pub filter: String,
    #[serde(default)]
    pub attributes: Vec<String>,
    /// Server-side size limit. `None` lets the server apply its default.
    #[serde(default)]
    pub size_limit: Option<u32>,
}

/// Single modification sent with [`crate::LdapClient::modify`].
///
/// `Delete { values: None }` removes the entire attribute; providing
/// `values` deletes only the listed ones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Modification {
    Add {
        attribute: String,
        values: Vec<String>,
    },
    Replace {
        attribute: String,
        values: Vec<String>,
    },
    Delete {
        attribute: String,
        #[serde(default)]
        values: Option<Vec<String>>,
    },
}

/// Structured schema summary returned by
/// [`crate::LdapClient::fetch_schema`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaInfo {
    pub subschema_dn: String,
    /// Backwards-compat: every attribute name we extracted (plain list).
    pub attribute_names: Vec<String>,
    /// Parsed attribute-type definitions (sorted by primary name).
    #[serde(default)]
    pub attribute_types: Vec<AttributeTypeDef>,
    /// Parsed objectClass definitions (sorted by name).
    pub object_classes: Vec<ObjectClassDef>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectClassKind {
    Abstract,
    #[default]
    Structural,
    Auxiliary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectClassDef {
    pub name: String,
    pub kind: ObjectClassKind,
    #[serde(default)]
    pub sup: Vec<String>,
    #[serde(default)]
    pub must: Vec<String>,
    #[serde(default)]
    pub may: Vec<String>,
}

/// Parsed RFC 4512 `attributeTypes` definition. Only the bits the UI
/// actually needs: name, OID, single/multi-valued, syntax, and the
/// SUP chain so the schema explorer can show inheritance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AttributeTypeDef {
    pub name: String,
    #[serde(default)]
    pub oid: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub sup: Option<String>,
    #[serde(default)]
    pub syntax: Option<String>,
    #[serde(default)]
    pub equality: Option<String>,
    #[serde(default)]
    pub ordering: Option<String>,
    #[serde(default)]
    pub substring: Option<String>,
    #[serde(default)]
    pub single_valued: bool,
    #[serde(default)]
    pub no_user_modification: bool,
    /// `userApplications` / `directoryOperation` /
    /// `distributedOperation` / `dSAOperation`.
    #[serde(default)]
    pub usage: Option<String>,
}

/// Effective MUST/MAY attribute lists for a given objectClass after
/// resolving the SUP chain. Returned by
/// [`crate::schema::resolve_must_may`].
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ResolvedClass {
    pub name: String,
    pub kind: ObjectClassKind,
    pub sup_chain: Vec<String>,
    pub must: Vec<String>,
    pub may: Vec<String>,
}

/// A single LDAP entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub dn: String,
    pub attributes: Vec<Attribute>,
}

/// A named attribute with one or more values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub values: Vec<AttributeValue>,
}

/// Attribute values are either text (LDAP `DirectoryString` &
/// `IA5String` families) or opaque bytes (`userCertificate`, photos…).
/// Binary bytes are encoded as base64 for the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "lowercase")]
pub enum AttributeValue {
    Text(String),
    /// Base64-encoded payload. The ctor `binary()` handles the encoding.
    Binary(String),
}

impl AttributeValue {
    /// Wrap raw bytes as a base64-encoded `Binary` value.
    #[must_use]
    pub fn binary(bytes: &[u8]) -> Self {
        Self::Binary(base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}

/// Short descriptor used by the DIT tree: DN, RDN, human label and a
/// best-effort hint on whether the entry has children (unknown = `None`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnLabel {
    pub dn: String,
    pub rdn: String,
    pub label: String,
    pub object_classes: Vec<String>,
    pub has_children: Option<bool>,
}

impl DnLabel {
    /// Build a label from an entry. `fallback` is used when no RDN value
    /// is extractable (e.g. a suffix like `dc=example,dc=org`).
    #[must_use]
    pub fn from_entry(entry: &Entry) -> Self {
        let rdn = rdn_of(&entry.dn).unwrap_or_default();
        let label = rdn_value(&rdn).unwrap_or_else(|| entry.dn.clone());
        let object_classes = entry
            .attributes
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("objectClass"))
            .map(|a| {
                a.values
                    .iter()
                    .filter_map(|v| match v {
                        AttributeValue::Text(t) => Some(t.clone()),
                        AttributeValue::Binary(_) => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            dn: entry.dn.clone(),
            rdn,
            label,
            object_classes,
            has_children: None,
        }
    }
}

/// Extract the left-most RDN from a DN. Returns `None` if the input is
/// empty. Does **not** unescape RFC 4514 escapes — callers that need
/// display-ready text should post-process.
fn rdn_of(dn: &str) -> Option<String> {
    // Split on the first unescaped comma.
    let mut escaped = false;
    for (idx, ch) in dn.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            ',' => return Some(dn[..idx].trim().to_string()),
            _ => {}
        }
    }
    if dn.is_empty() {
        None
    } else {
        Some(dn.trim().to_string())
    }
}

/// Return the `value` side of an `attr=value` RDN. Handles the case of
/// compound RDNs (`cn=foo+uid=bar`) by keeping only the first atom.
fn rdn_value(rdn: &str) -> Option<String> {
    let first_atom = rdn.split('+').next().unwrap_or(rdn);
    first_atom
        .split_once('=')
        .map(|(_, v)| v.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdn_of_returns_first_component() {
        assert_eq!(
            rdn_of("cn=Alice,ou=People,dc=ex,dc=org").as_deref(),
            Some("cn=Alice")
        );
        assert_eq!(rdn_of("dc=ex,dc=org").as_deref(), Some("dc=ex"));
        assert_eq!(rdn_of("").as_deref(), None);
    }

    #[test]
    fn rdn_of_handles_escaped_comma() {
        assert_eq!(
            rdn_of(r"cn=Doe\, John,ou=People,dc=ex").as_deref(),
            Some(r"cn=Doe\, John"),
        );
    }

    #[test]
    fn rdn_value_splits_on_equals() {
        assert_eq!(rdn_value("cn=Alice"), Some("Alice".into()));
        assert_eq!(rdn_value("cn=Alice+uid=alice"), Some("Alice".into()));
        assert_eq!(rdn_value("broken"), None);
    }

    #[test]
    fn binary_value_is_base64() {
        let v = AttributeValue::binary(b"hi");
        assert!(matches!(v, AttributeValue::Binary(ref s) if s == "aGk="));
    }
}
