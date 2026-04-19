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
