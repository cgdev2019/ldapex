use ldap3::LdapError as RawLdapError;
use serde::Serialize;
use thiserror::Error;

/// Errors surfaced by the core crate.
///
/// Variants are coarse on purpose: they map to stable UI messages.
/// The `#[serde(tag = "kind", content = "message")]` shape is what the
/// frontend receives through Tauri commands.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum LdapexError {
    /// Simple or SASL bind rejected by the server (LDAP result code 49).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// The target DN does not exist (LDAP result code 32).
    #[error("no such object: {0}")]
    NoSuchObject(String),

    /// The client tried to use the session before binding.
    #[error("not connected")]
    NotConnected,

    /// LDAP protocol error returned by the server. The inner message is
    /// the diagnostic text from the server.
    #[error("LDAP protocol error (code {code}): {message}")]
    Protocol { code: u32, message: String },

    /// TLS negotiation or certificate verification failure.
    #[error("TLS error: {0}")]
    Tls(String),

    /// Transport-level I/O error (connection reset, timeout, DNS, …).
    #[error("I/O error: {0}")]
    Io(String),

    /// Input rejected before reaching the server (malformed DN, bad URL).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Anything the caller should not have to explain to the user.
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, LdapexError>;

impl From<RawLdapError> for LdapexError {
    fn from(err: RawLdapError) -> Self {
        match err {
            RawLdapError::LdapResult { result } => Self::from_result_code(result.rc, result.text),
            RawLdapError::EmptyUnixPath => Self::InvalidInput("empty unix socket path".into()),
            RawLdapError::PortInUnixPath => {
                Self::InvalidInput("unix path must not carry a port".into())
            }
            RawLdapError::FilterParsing => Self::InvalidInput("invalid LDAP filter".into()),
            RawLdapError::ResultRecv { .. } | RawLdapError::OpSend { .. } => {
                Self::Io(err.to_string())
            }
            RawLdapError::Io { source, .. } => Self::Io(source.to_string()),
            RawLdapError::UrlParsing { source } => Self::InvalidInput(source.to_string()),
            RawLdapError::UnknownScheme(s) => {
                Self::InvalidInput(format!("unknown URL scheme: {s}"))
            }
            other => Self::Internal(other.to_string()),
        }
    }
}

impl LdapexError {
    /// Map an LDAP result code (RFC 4511 §4.1.9) to a typed variant.
    /// See <https://www.rfc-editor.org/rfc/rfc4511#appendix-A>.
    pub(crate) fn from_result_code(rc: u32, diagnostic: String) -> Self {
        match rc {
            0 => Self::Internal(format!("unexpected success treated as error: {diagnostic}")),
            32 => Self::NoSuchObject(diagnostic),
            49 => Self::InvalidCredentials,
            _ => Self::Protocol {
                code: rc,
                message: diagnostic,
            },
        }
    }
}
