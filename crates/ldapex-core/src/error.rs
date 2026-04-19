use serde::Serialize;
use thiserror::Error;

/// Errors surfaced by the core crate.
///
/// The variants are kept coarse on purpose: they map to stable UI
/// messages. Richer context lives in the wrapped `source`.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum LdapexError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("no such object: {0}")]
    NoSuchObject(String),

    #[error("LDAP protocol error: {0}")]
    Protocol(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, LdapexError>;
