use ldapex_core::{
    entries_to_ldif, schema::resolve_must_may, Attribute, ConnectOptions, ConnectionProfile,
    DnLabel, Entry, LdapClient, LdapexError, Modification, ResolvedClass, Result as CoreResult,
    SchemaInfo, SearchParams, SearchScope, TlsMode,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::profiles::{ProfileError, ProfileManager};

/// Shared application state. A single LDAP session at a time is enough
/// for the MVP; multi-session can come later by swapping this for a
/// `HashMap<ProfileId, LdapClient>`.
pub struct AppState {
    session: Mutex<Option<LdapClient>>,
    profiles: ProfileManager,
}

impl AppState {
    pub fn new() -> Result<Self, ProfileError> {
        Ok(Self {
            session: Mutex::new(None),
            profiles: ProfileManager::new()?,
        })
    }
}

fn profile_err(err: ProfileError) -> LdapexError {
    LdapexError::Internal(err.to_string())
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

// -------------------- Session commands --------------------

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

/// Resolve the effective MUST/MAY for one or more objectClasses. The
/// frontend passes the full schema (so we don't refetch on every
/// click) plus the names to resolve; the result is one
/// `ResolvedClass` per name. Unknown names are skipped silently.
#[derive(Debug, Deserialize)]
pub struct ResolveClassesInput {
    pub schema: SchemaInfo,
    pub names: Vec<String>,
}

#[tauri::command]
pub fn schema_resolve_classes(input: ResolveClassesInput) -> Vec<ResolvedClass> {
    input
        .names
        .iter()
        .filter_map(|n| resolve_must_may(n, &input.schema.object_classes))
        .collect()
}

/// Export a DN (and optionally its subtree) to a single RFC 2849 LDIF
/// blob. `scope` defaults to `base`; the UI passes `subtree` for the
/// "export this branch" flow.
#[derive(Debug, Deserialize)]
pub struct ExportLdifInput {
    pub base_dn: String,
    #[serde(default = "default_export_scope")]
    pub scope: SearchScope,
}

fn default_export_scope() -> SearchScope {
    SearchScope::Base
}

#[derive(Debug, Serialize)]
pub struct LdifExportResult {
    pub ldif: String,
    pub entry_count: usize,
}

#[tauri::command]
pub async fn ldap_export_ldif(
    state: State<'_, AppState>,
    input: ExportLdifInput,
) -> CoreResult<LdifExportResult> {
    let guard = state.session.lock().await;
    let client = guard.as_ref().ok_or(LdapexError::NotConnected)?;
    let entries = client
        .search(SearchParams {
            base_dn: input.base_dn,
            scope: input.scope,
            filter: "(objectClass=*)".into(),
            attributes: vec![],
            size_limit: None,
        })
        .await?;
    let ldif = entries_to_ldif(&entries);
    Ok(LdifExportResult {
        ldif,
        entry_count: entries.len(),
    })
}

// -------------------- Profile commands --------------------

/// Summary returned to the UI. The `has_saved_password` flag lets the
/// picker decide whether to prompt for a password before connecting.
#[derive(Debug, Serialize)]
pub struct ProfileSummary {
    #[serde(flatten)]
    pub profile: ConnectionProfile,
    pub has_saved_password: bool,
}

fn summarise(profile: ConnectionProfile) -> ProfileSummary {
    let has_saved_password = profile.password.is_some();
    ProfileSummary {
        profile,
        has_saved_password,
    }
}

#[tauri::command]
pub fn profile_list(state: State<'_, AppState>) -> CoreResult<Vec<ProfileSummary>> {
    let profiles = state.profiles.list().map_err(profile_err)?;
    Ok(profiles.into_iter().map(summarise).collect())
}

#[tauri::command]
pub fn profile_save(
    state: State<'_, AppState>,
    profile: ConnectionProfile,
) -> CoreResult<ProfileSummary> {
    let saved = state.profiles.save(profile).map_err(profile_err)?;
    Ok(summarise(saved))
}

#[tauri::command]
pub fn profile_delete(state: State<'_, AppState>, id: String) -> CoreResult<()> {
    state.profiles.delete(&id).map_err(profile_err)
}

#[tauri::command]
pub fn profile_export(state: State<'_, AppState>) -> CoreResult<String> {
    state.profiles.export_json().map_err(profile_err)
}

#[tauri::command]
pub fn profile_import(state: State<'_, AppState>, json: String) -> CoreResult<Vec<ProfileSummary>> {
    let merged = state.profiles.import_json(&json).map_err(profile_err)?;
    Ok(merged.into_iter().map(summarise).collect())
}

/// Connect using a saved profile. `password` is optional:
/// - if provided, it overrides the one stored in the profile (and, when
///   `remember = true`, overwrites it for subsequent connections);
/// - if absent, the password stored in the profile is used.
#[derive(Debug, Deserialize)]
pub struct ProfileConnectInput {
    pub id: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub remember: bool,
}

#[tauri::command]
pub async fn profile_connect(
    state: State<'_, AppState>,
    input: ProfileConnectInput,
) -> CoreResult<ConnectionProfile> {
    let mut profile = state
        .profiles
        .list()
        .map_err(profile_err)?
        .into_iter()
        .find(|p| p.id == input.id)
        .ok_or_else(|| LdapexError::Internal(format!("profile not found: {}", input.id)))?;

    let password = input
        .password
        .as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
        .or_else(|| profile.password.clone())
        .ok_or(LdapexError::InvalidCredentials)?;

    let options = ConnectOptions {
        url: profile.url.clone(),
        tls: profile.tls,
        timeout_secs: profile.timeout_secs.or(Some(30)),
    };
    let client = LdapClient::connect(options).await?;
    client.simple_bind(&profile.bind_dn, &password).await?;

    if input.remember
        && input
            .password
            .as_deref()
            .filter(|s| !s.is_empty())
            .is_some()
    {
        profile.password = Some(password);
        let stored = state.profiles.save(profile.clone()).map_err(profile_err)?;
        profile = stored;
    }

    *state.session.lock().await = Some(client);
    Ok(profile)
}
