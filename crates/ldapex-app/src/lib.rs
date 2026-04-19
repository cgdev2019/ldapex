//! Ldapex Tauri shell. Keeps the `main` binary minimal so that the
//! entry point can be reused by integration tests and, eventually, a
//! mobile target.

mod commands;
mod profiles;

use commands::AppState;
use tracing_subscriber::EnvFilter;

/// Entry point shared between `main.rs` and any alternative binary.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let state = AppState::new().expect("initialise profile storage");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::ldap_connect,
            commands::ldap_disconnect,
            commands::ldap_list_children,
            commands::ldap_read_entry,
            commands::ldap_search,
            commands::ldap_modify,
            commands::ldap_add,
            commands::ldap_delete,
            commands::ldap_rename,
            commands::ldap_fetch_schema,
            commands::profile_list,
            commands::profile_save,
            commands::profile_delete,
            commands::profile_export,
            commands::profile_import,
            commands::profile_connect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ldapex");
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,ldapex=debug"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
