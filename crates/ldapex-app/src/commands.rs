use ldapex_core::{
    Attribute, ConnectOptions, DnLabel, Entry, LdapClient, LdapexError, Modification,
    Result as CoreResult, SchemaInfo, SearchParams, TlsMode,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

/// Shared application state. A single LDAP session at a time is enough
/// for the MVP; multi-session can come later by swapping this for a
/// `HashMap<ProfileId, LdapClient>`.
#[derive(Default)]
pub struct AppState {
    session: Mutex<Option<LdapClient>>,
}

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

/// Input for the `ldap_connect` command.
#[derive(Debug, Deserialize)]
pub struct ConnectInput {
    pub url: String,
    pub bind_dn: String,
    pub password: String,
    #[serde(default)]
    pub tls: TlsMode,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[tauri::command]
pub async fn ldap_connect(state: State<'_, AppState>, input: ConnectInput) -> CoreResult<()> {
    let options = ConnectOptions {
        url: input.url,
        tls: input.tls,
        timeout_secs: input.timeout_secs.or(Some(30)),
    };
    let client = LdapClient::connect(options).await?;
    client.simple_bind(&input.bind_dn, &input.password).await?;

    // Replace any previous session. Dropping the previous `LdapClient`
    // sends an unbind on the underlying handle, which is enough for
    // MVP; we do not block on it.
    *state.session.lock().await = Some(client);
    Ok(())
}

#[tauri::command]
pub async fn ldap_disconnect(state: State<'_, AppState>) -> CoreResult<()> {
    let _ = state.session.lock().await.take();
    Ok(())
}

#[tauri::command]
pub async fn ldap_list_children(
    state: State<'_, AppState>,
    base_dn: String,
) -> CoreResult<Vec<DnLabel>> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.list_children(&base_dn).await
}

#[tauri::command]
pub async fn ldap_read_entry(state: State<'_, AppState>, dn: String) -> CoreResult<Entry> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.read_entry(&dn).await
}

#[tauri::command]
pub async fn ldap_search(
    state: State<'_, AppState>,
    params: SearchParams,
) -> CoreResult<Vec<Entry>> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.search(params).await
}

#[tauri::command]
pub async fn ldap_modify(
    state: State<'_, AppState>,
    dn: String,
    mods: Vec<Modification>,
) -> CoreResult<()> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.modify(&dn, &mods).await
}

#[tauri::command]
pub async fn ldap_add(
    state: State<'_, AppState>,
    dn: String,
    attributes: Vec<Attribute>,
) -> CoreResult<()> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.add(&dn, &attributes).await
}

#[tauri::command]
pub async fn ldap_delete(state: State<'_, AppState>, dn: String) -> CoreResult<()> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.delete(&dn).await
}

#[derive(Debug, Deserialize)]
pub struct RenameInput {
    pub dn: String,
    pub new_rdn: String,
    #[serde(default)]
    pub new_parent: Option<String>,
    #[serde(default = "default_true")]
    pub delete_old_rdn: bool,
}

fn default_true() -> bool {
    true
}

#[tauri::command]
pub async fn ldap_rename(state: State<'_, AppState>, input: RenameInput) -> CoreResult<()> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client
        .rename(
            &input.dn,
            &input.new_rdn,
            input.new_parent.as_deref(),
            input.delete_old_rdn,
        )
        .await
}

#[tauri::command]
pub async fn ldap_fetch_schema(state: State<'_, AppState>) -> CoreResult<SchemaInfo> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    client.fetch_schema().await
}
