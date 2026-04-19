use serde::Serialize;

/// Shape returned by the bootstrap `ping` command. Mirrors the type
/// exposed to the frontend in `frontend/src/lib/bridge.ts`.
#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub core_version: &'static str,
    pub app_version: &'static str,
}

#[tauri::command]
pub fn ping() -> PingResponse {
    PingResponse {
        message: ldapex_core::ping(),
        core_version: ldapex_core::VERSION,
        app_version: env!("CARGO_PKG_VERSION"),
    }
}
