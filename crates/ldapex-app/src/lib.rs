//! Ldapex Tauri shell. Keeps the `main` binary minimal so that the
//! entry point can be reused by integration tests and, eventually, a
//! mobile target.

mod commands;
mod logging;
mod profiles;

use commands::AppState;

/// Entry point shared between `main.rs` and any alternative binary.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Hold on to the appender guard for the lifetime of the process so
    // the background log-writer thread keeps flushing.
    let _log_guard = logging::init();

    let state = AppState::new().expect("initialise profile storage");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
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
            commands::ldap_export_ldif,
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
