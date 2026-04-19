//! RFC 2849 LDIF serializer — minimal but spec-friendly.
//!
//! The goal is to produce text that round-trips through `ldapadd` /
//! `ldapmodify` without surprises:
//!
//! - attribute names stay as-is
//! - ASCII text values with no "unsafe" character go on a single line
//!   (`attr: value`)
//! - any other text value (leading space, tab, colon, non-ASCII, etc.)
//!   is base64-encoded (`attr:: base64...`)
//! - binary values are always base64-encoded
//! - entries are separated by a blank line
//!
//! We intentionally skip line folding (RFC 2849 §8 rule 2): all the
//! parsers we have tried tolerate long lines and it keeps diffs human.

use base64::Engine as _;

use crate::types::{AttributeValue, Entry};

/// Serialise a slice of entries to an LDIF blob.
#[must_use]
pub fn entries_to_ldif(entries: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("version: 1\n\n");
    for (i, entry) in entries.iter().enumerate() {
        write_entry(&mut out, entry);
        if i + 1 < entries.len() {
            out.push('\n');
        }
    }
    out
}

fn write_entry(out: &mut String, entry: &Entry) {
    write_line(
        out,
        "dn",
        entry.dn.as_bytes(),
        is_text_safe(entry.dn.as_bytes()),
    );
    for attr in &entry.attributes {
        for value in &attr.values {
            match value {
                AttributeValue::Text(s) => {
                    let bytes = s.as_bytes();
                    write_line(out, &attr.name, bytes, is_text_safe(bytes));
                }
                AttributeValue::Binary(b64) => {
                    // Already base64-encoded in the domain type.
                    out.push_str(&attr.name);
                    out.push_str(":: ");
                    out.push_str(b64);
                    out.push('\n');
                }
            }
        }
    }
}

fn write_line(out: &mut String, attr: &str, value: &[u8], safe: bool) {
    if safe {
        out.push_str(attr);
        out.push_str(": ");
        // SAFETY: we only come here when `is_text_safe` accepted the
        // bytes, which enforces ASCII.
        out.push_str(std::str::from_utf8(value).unwrap_or(""));
        out.push('\n');
    } else {
        out.push_str(attr);
        out.push_str(":: ");
        out.push_str(&base64::engine::general_purpose::STANDARD.encode(value));
        out.push('\n');
    }
}

/// Per RFC 2849 §8 rule 4: a value is SAFE-STRING iff it is non-empty,
/// starts with a SAFE-INIT-CHAR (no NUL, LF, CR, space, colon, `<`) and
/// every subsequent char is SAFE-CHAR (no NUL, LF, CR). We also refuse
/// non-ASCII bytes so the output stays diff-friendly.
fn is_text_safe(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let first = bytes[0];
    if first == 0
        || first == b'\n'
        || first == b'\r'
        || first == b' '
        || first == b':'
        || first == b'<'
    {
        return false;
    }
    for &b in bytes {
        if !(32..=126).contains(&b) {
            return false;
        }
    }
    // trailing space is also discouraged
    bytes.last() != Some(&b' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Attribute, AttributeValue, Entry};

    fn sample() -> Entry {
        Entry {
            dn: "uid=alice,ou=People,dc=ldapex,dc=test".into(),
            attributes: vec![
                Attribute {
                    name: "objectClass".into(),
                    values: vec![
                        AttributeValue::Text("top".into()),
                        AttributeValue::Text("inetOrgPerson".into()),
                    ],
                },
                Attribute {
                    name: "cn".into(),
                    values: vec![AttributeValue::Text("Alice".into())],
                },
            ],
        }
    }

    #[test]
    fn writes_basic_entry() {
        let out = entries_to_ldif(&[sample()]);
        assert!(out.starts_with("version: 1\n\n"));
        assert!(out.contains("dn: uid=alice,ou=People,dc=ldapex,dc=test\n"));
        assert!(out.contains("objectClass: top\n"));
        assert!(out.contains("objectClass: inetOrgPerson\n"));
        assert!(out.contains("cn: Alice\n"));
    }

    #[test]
    fn base64_encodes_unsafe_values() {
        let entry = Entry {
            dn: "cn=Acme Corp,dc=ex".into(),
            attributes: vec![
                Attribute {
                    name: "description".into(),
                    // leading space → must be base64-encoded
                    values: vec![AttributeValue::Text(" starts with space".into())],
                },
                Attribute {
                    name: "givenName".into(),
                    // non-ASCII → must be base64-encoded
                    values: vec![AttributeValue::Text("Héloïse".into())],
                },
                Attribute {
                    name: "userCertificate;binary".into(),
                    values: vec![AttributeValue::Binary("deadbeef==".into())],
                },
            ],
        };
        let out = entries_to_ldif(&[entry]);
        assert!(out.contains("description::"));
        assert!(!out.contains("description: "));
        assert!(out.contains("givenName::"));
        assert!(out.contains("userCertificate;binary:: deadbeef==\n"));
    }

    #[test]
    fn separates_entries_with_blank_line() {
        let out = entries_to_ldif(&[sample(), sample()]);
        // Expect one empty line between two entries.
        let body = out.trim_start_matches("version: 1\n\n");
        assert!(
            body.contains("\n\ndn:"),
            "missing blank separator in {body}"
        );
    }

    #[test]
    fn safe_string_rules() {
        assert!(is_text_safe(b"hello"));
        assert!(is_text_safe(b"cn=Alice"));
        assert!(!is_text_safe(b" leading"));
        assert!(!is_text_safe(b":colon"));
        assert!(!is_text_safe(b"with\nlf"));
        assert!(!is_text_safe("héloïse".as_bytes()));
        assert!(!is_text_safe(b"trailing "));
    }
}
