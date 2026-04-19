import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { _ } from 'svelte-i18n';

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
  const t = get(_);
  if (err && typeof err === 'object' && 'kind' in err) {
    const e = err as LdapexError;
    switch (e.kind) {
      case 'invalid_credentials':
        return t('errors.invalid_credentials');
      case 'not_connected':
        return t('errors.not_connected');
      case 'no_such_object':
        return t('errors.no_such_object', { values: { message: String(e.message) } });
      case 'protocol':
        if (typeof e.message === 'object' && e.message !== null) {
          return t('errors.protocol_with_code', {
            values: { code: e.message.code, message: e.message.message }
          });
        }
        return t('errors.protocol', { values: { message: String(e.message) } });
      case 'tls':
        return t('errors.tls', { values: { message: String(e.message) } });
      case 'io':
        return t('errors.io', { values: { message: String(e.message) } });
      case 'invalid_input':
        return t('errors.invalid_input', { values: { message: String(e.message) } });
      case 'internal':
      default:
        return t('errors.internal', {
          values: { message: String(e.message ?? t('errors.unknown')) }
        });
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

export interface ExportLdifInput {
  base_dn: string;
  scope?: SearchScope;
}

export interface LdifExportResult {
  ldif: string;
  entry_count: number;
}

export async function ldapExportLdif(input: ExportLdifInput): Promise<LdifExportResult> {
  return invoke<LdifExportResult>('ldap_export_ldif', { input });
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
  /**
   * Bind password persisted in `~/.ldapex/profiles.toml`. Omit / leave
   * empty to force a password prompt at connect time.
   *
   * Stored in plain text — the backend chmods the file to 0600 on Unix.
   */
  password?: string | null;
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

export async function profileSave(profile: ConnectionProfile): Promise<ProfileSummary> {
  return invoke<ProfileSummary>('profile_save', { profile });
}

export async function profileDelete(id: string): Promise<void> {
  await invoke('profile_delete', { id });
}

export async function profileExport(): Promise<string> {
  return invoke<string>('profile_export');
}

export async function profileImport(json: string): Promise<ProfileSummary[]> {
  return invoke<ProfileSummary[]>('profile_import', { json });
}

export async function profileConnect(input: ProfileConnectInput): Promise<ConnectionProfile> {
  return invoke<ConnectionProfile>('profile_connect', { input });
}
