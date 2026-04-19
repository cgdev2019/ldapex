//! Ldapex Tauri shell. Keeps the `main` binary minimal so that the
//! entry point can be reused by integration tests and, eventually, a
//! mobile target.

mod commands;

use tracing_subscriber::EnvFilter;

/// Entry point shared between `main.rs` and any alternative binary.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running Ldapex");
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,ldapex=debug"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
