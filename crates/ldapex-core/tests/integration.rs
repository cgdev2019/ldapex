//! End-to-end tests against a real `OpenLDAP` instance.
//!
//! Gated behind the `integration-tests` feature so `cargo test` stays
//! offline by default. To run these tests:
//!
//! ```sh
//! docker compose -f docker/openldap/docker-compose.yml up -d
//! cargo test -p ldapex-core --features integration-tests -- --nocapture
//! ```
//!
//! The expected seed lives in `docker/openldap/seed.ldif`.

#![cfg(feature = "integration-tests")]

use ldapex_core::{AttributeValue, ConnectOptions, LdapClient, LdapexError};

const URL: &str = "ldap://127.0.0.1:3389";
const ADMIN_DN: &str = "cn=admin,dc=ldapex,dc=test";
const ADMIN_PW: &str = "admin";
const BASE: &str = "dc=ldapex,dc=test";

async fn connected_client() -> LdapClient {
    let client = LdapClient::connect(ConnectOptions::new(URL))
        .await
        .expect("connect");
    client.simple_bind(ADMIN_DN, ADMIN_PW).await.expect("bind");
    client
}

#[tokio::test]
async fn bind_and_list_root_children() {
    let client = connected_client().await;
    let children = client.list_children(BASE).await.expect("list_children");

    let names: Vec<&str> = children.iter().map(|c| c.rdn.as_str()).collect();
    assert!(
        names.contains(&"ou=People"),
        "missing ou=People, got {names:?}"
    );
    assert!(
        names.contains(&"ou=Groups"),
        "missing ou=Groups, got {names:?}"
    );
}

#[tokio::test]
async fn read_alice_entry() {
    let client = connected_client().await;
    let entry = client
        .read_entry("uid=alice,ou=People,dc=ldapex,dc=test")
        .await
        .expect("read_entry");

    assert_eq!(entry.dn, "uid=alice,ou=People,dc=ldapex,dc=test");
    let mail = entry
        .attributes
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("mail"))
        .expect("mail attribute");
    assert!(matches!(
        &mail.values[0],
        AttributeValue::Text(s) if s == "alice@ldapex.test"
    ));
}

#[tokio::test]
async fn wrong_password_is_invalid_credentials() {
    let client = LdapClient::connect(ConnectOptions::new(URL))
        .await
        .expect("connect");
    let err = client.simple_bind(ADMIN_DN, "wrong").await.unwrap_err();
    assert!(
        matches!(err, LdapexError::InvalidCredentials),
        "got {err:?}"
    );
}

#[tokio::test]
async fn missing_dn_is_no_such_object() {
    let client = connected_client().await;
    let err = client
        .read_entry("uid=ghost,ou=People,dc=ldapex,dc=test")
        .await
        .unwrap_err();
    assert!(matches!(err, LdapexError::NoSuchObject(_)), "got {err:?}");
}
