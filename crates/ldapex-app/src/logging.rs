//! Tracing setup for the Tauri app.
//!
//! Two sinks are wired:
//!
//! - stdout (kept from Phase 0, useful when launching from a terminal),
//! - a daily-rotated file under `~/.ldapex/logs/ldapex.log` so users can
//!   attach logs to bug reports without re-running with `RUST_LOG`.
//!
//! Every line written to either sink first goes through
//! [`RedactingWriter`], a tiny `MakeWriter` that replaces things like
//! `password=hunter2` or `password: hunter2` with `password=***`. The
//! current `#[instrument(skip(password))]` annotations on
//! [`ldapex_core::client::LdapClient::simple_bind`] already prevent the
//! span machinery from recording the secret; this writer is the
//! belt-and-braces guard for any future log line that forgets to do so.
//!
//! The returned [`tracing_appender::non_blocking::WorkerGuard`] must be
//! kept alive for the lifetime of the process — drop it and the
//! background writer thread is torn down, swallowing any buffered line.

use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::OnceLock,
};

use regex::bytes::Regex;
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling,
};
use tracing_subscriber::{
    fmt::writer::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::profiles::{config_file, ProfileError};

/// Subdirectory of `~/.ldapex/` that holds rolling log files.
pub const LOGS_DIR: &str = "logs";

/// Filename prefix passed to `tracing_appender::rolling::daily`. The
/// final files are e.g. `ldapex.log.2025-12-01`.
pub const LOG_FILE_PREFIX: &str = "ldapex.log";

/// Compile the redaction regex once. Pattern matches the attribute name
/// `password` (case-insensitive) followed by `:` or `=`, optional
/// whitespace, then a non-whitespace value to mask.
fn redaction_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(password\s*[:=]\s*)\S+").expect("redaction regex compiles"))
}

/// Apply the redaction once over `buf`. Returns a freshly owned `Vec`
/// only when a substitution actually happened, so the common path stays
/// allocation-free.
fn redact(buf: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    redaction_regex().replace_all(buf, &b"$1***"[..])
}

/// `MakeWriter` adapter that pipes every line through [`redact`] before
/// forwarding it to the inner writer.
#[derive(Clone)]
pub struct RedactingMakeWriter<M> {
    inner: M,
}

impl<M> RedactingMakeWriter<M> {
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<'a, M> MakeWriter<'a> for RedactingMakeWriter<M>
where
    M: MakeWriter<'a>,
{
    type Writer = RedactingWriter<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: self.inner.make_writer(),
        }
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        RedactingWriter {
            inner: self.inner.make_writer_for(meta),
        }
    }
}

/// `Write` wrapper produced by [`RedactingMakeWriter`]. Each `write`
/// runs the redaction regex on the buffer; we always claim every byte
/// of the input was consumed because callers (the tracing fmt layer)
/// hand us a fully formatted record at a time.
pub struct RedactingWriter<W> {
    inner: W,
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let cleaned = redact(buf);
        self.inner.write_all(&cleaned)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Resolve `~/.ldapex/logs/`, creating it if needed.
pub fn logs_dir() -> Result<PathBuf, ProfileError> {
    let cfg = config_file()?; // ~/.ldapex/profiles.toml
    let parent = cfg
        .parent()
        .ok_or_else(|| ProfileError::Io(io::Error::new(io::ErrorKind::Other, "no home")))?;
    let dir = parent.join(LOGS_DIR);
    fs::create_dir_all(&dir).map_err(ProfileError::Io)?;
    Ok(dir)
}

/// Initialise stdout + file tracing. Returns the worker guard that
/// flushes the async file appender on drop.
///
/// Both layers go through [`RedactingMakeWriter`] so the password
/// redaction happens consistently regardless of where the line lands.
pub fn init() -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,ldapex=debug"));

    let stdout_layer =
        tracing_subscriber::fmt::layer().with_writer(RedactingMakeWriter::new(io::stdout));

    // File sink (best-effort: if `~/.ldapex/logs/` cannot be created we
    // silently fall back to stdout-only).
    match logs_dir() {
        Ok(dir) => {
            let appender = rolling::daily(dir, LOG_FILE_PREFIX);
            let (nb, guard): (NonBlocking, WorkerGuard) = tracing_appender::non_blocking(appender);
            let file_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_writer(RedactingMakeWriter::new(nb));
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .try_init();
            Some(guard)
        }
        Err(_) => {
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .try_init();
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn redacts_password_with_equals() {
        let out = redact(b"bind password=hunter2 done");
        assert_eq!(out.as_ref(), b"bind password=*** done");
    }

    #[test]
    fn redacts_password_with_colon_and_spaces() {
        let out = redact(b"Password : sup3r-secret");
        assert_eq!(out.as_ref(), b"Password : ***");
    }

    #[test]
    fn untouched_when_no_password_token() {
        let out = redact(b"connecting to ldap://srv");
        assert_eq!(out.as_ref(), b"connecting to ldap://srv");
    }
}
