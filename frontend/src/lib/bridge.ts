import { invoke } from '@tauri-apps/api/core';

/**
 * Types below mirror `crates/ldapex-core/src/{types,client,error}.rs`
 * and `crates/ldapex-app/src/commands.rs`. They are kept in sync by
 * hand for now; Phase 1 task "Câbler bridge" wired a manual mirror,
 * Phase 4 can switch to `ts-rs`/specta if it becomes painful.
 */

// ---------- Core types ----------

export type TlsMode = 'none' | 'start_tls' | 'ldaps';

export interface AttributeValueText {
  kind: 'text';
  data: string;
}

export interface AttributeValueBinary {
  kind: 'binary';
  data: string; // base64
}

export type AttributeValue = AttributeValueText | AttributeValueBinary;

export interface Attribute {
  name: string;
  values: AttributeValue[];
}

export interface Entry {
  dn: string;
  attributes: Attribute[];
}

export interface DnLabel {
  dn: string;
  rdn: string;
  label: string;
  object_classes: string[];
  has_children: boolean | null;
}

// ---------- Errors ----------

export type LdapexErrorKind =
  | 'invalid_credentials'
  | 'no_such_object'
  | 'not_connected'
  | 'protocol'
  | 'tls'
  | 'io'
  | 'invalid_input'
  | 'internal';

export interface LdapexError {
  kind: LdapexErrorKind;
  message?: string | { code: number; message: string };
}

export function formatError(err: unknown): string {
  if (err && typeof err === 'object' && 'kind' in err) {
    const e = err as LdapexError;
    switch (e.kind) {
      case 'invalid_credentials':
        return 'Identifiants invalides.';
      case 'not_connected':
        return 'Session LDAP fermée.';
      case 'no_such_object':
        return `Entrée introuvable : ${String(e.message)}`;
      case 'protocol':
        if (typeof e.message === 'object') {
          return `Erreur LDAP ${e.message.code} : ${e.message.message}`;
        }
        return `Erreur LDAP : ${String(e.message)}`;
      case 'tls':
        return `TLS : ${String(e.message)}`;
      case 'io':
        return `I/O : ${String(e.message)}`;
      case 'invalid_input':
        return `Entrée invalide : ${String(e.message)}`;
      case 'internal':
      default:
        return `Erreur interne : ${String(e.message ?? 'inconnue')}`;
    }
  }
  return err instanceof Error ? err.message : String(err);
}

// ---------- Commands ----------

export interface PingResponse {
  message: string;
  core_version: string;
  app_version: string;
}

export async function ping(): Promise<PingResponse> {
  return invoke<PingResponse>('ping');
}

export interface ConnectInput {
  url: string;
  bind_dn: string;
  password: string;
  tls?: TlsMode;
  timeout_secs?: number;
}

export async function ldapConnect(input: ConnectInput): Promise<void> {
  await invoke('ldap_connect', { input });
}

export async function ldapDisconnect(): Promise<void> {
  await invoke('ldap_disconnect');
}

export async function ldapListChildren(baseDn: string): Promise<DnLabel[]> {
  return invoke<DnLabel[]>('ldap_list_children', { baseDn });
}

export async function ldapReadEntry(dn: string): Promise<Entry> {
  return invoke<Entry>('ldap_read_entry', { dn });
}

// ---------- Phase 2 types ----------

export type SearchScope = 'base' | 'one_level' | 'subtree';

export interface SearchParams {
  base_dn: string;
  scope: SearchScope;
  filter: string;
  attributes?: string[];
  size_limit?: number | null;
}

export type Modification =
  | { op: 'add'; attribute: string; values: string[] }
  | { op: 'replace'; attribute: string; values: string[] }
  | { op: 'delete'; attribute: string; values?: string[] | null };

export type ObjectClassKind = 'abstract' | 'structural' | 'auxiliary';

export interface ObjectClassDef {
  name: string;
  kind: ObjectClassKind;
  sup: string[];
  must: string[];
  may: string[];
}

export interface SchemaInfo {
  subschema_dn: string;
  attribute_names: string[];
  object_classes: ObjectClassDef[];
}

// ---------- Phase 2 commands ----------

export async function ldapSearch(params: SearchParams): Promise<Entry[]> {
  return invoke<Entry[]>('ldap_search', { params });
}

export async function ldapModify(dn: string, mods: Modification[]): Promise<void> {
  await invoke('ldap_modify', { dn, mods });
}

export async function ldapAdd(dn: string, attributes: Attribute[]): Promise<void> {
  await invoke('ldap_add', { dn, attributes });
}

export async function ldapDelete(dn: string): Promise<void> {
  await invoke('ldap_delete', { dn });
}

export interface RenameInput {
  dn: string;
  new_rdn: string;
  new_parent?: string | null;
  delete_old_rdn?: boolean;
}

export async function ldapRename(input: RenameInput): Promise<void> {
  await invoke('ldap_rename', { input });
}

export async function ldapFetchSchema(): Promise<SchemaInfo> {
  return invoke<SchemaInfo>('ldap_fetch_schema');
}

// ---------- Phase 3 types ----------

export interface ConnectionProfile {
  id: string;
  name: string;
  url: string;
  bind_dn: string;
  base_dn: string;
  tls: TlsMode;
  timeout_secs?: number | null;
  save_password: boolean;
}

export interface ProfileSummary extends ConnectionProfile {
  has_saved_password: boolean;
}

export interface ProfileConnectInput {
  id: string;
  password?: string;
  remember?: boolean;
}

// ---------- Phase 3 commands ----------

export async function profileList(): Promise<ProfileSummary[]> {
  return invoke<ProfileSummary[]>('profile_list');
}

export async function profileSave(profile: ConnectionProfile): Promise<ConnectionProfile> {
  return invoke<ConnectionProfile>('profile_save', { profile });
}

export async function profileDelete(id: string): Promise<void> {
  await invoke('profile_delete', { id });
}

export async function profileSetPassword(id: string, password: string): Promise<void> {
  await invoke('profile_set_password', { id, password });
}

export async function profileClearPassword(id: string): Promise<void> {
  await invoke('profile_clear_password', { id });
}

export async function profileExport(): Promise<string> {
  return invoke<string>('profile_export');
}

export async function profileImport(json: string): Promise<ConnectionProfile[]> {
  return invoke<ConnectionProfile[]>('profile_import', { json });
}

export async function profileConnect(input: ProfileConnectInput): Promise<ConnectionProfile> {
  return invoke<ConnectionProfile>('profile_connect', { input });
}
