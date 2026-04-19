import { invoke } from '@tauri-apps/api/core';

/**
 * Mirror of `PingResponse` in `crates/ldapex-app/src/commands.rs`.
 * When we add `ts-rs` (Phase 0 "bridge" task), these types will be
 * generated automatically; for now we keep them in sync by hand.
 */
export interface PingResponse {
  message: string;
  core_version: string;
  app_version: string;
}

export async function ping(): Promise<PingResponse> {
  return invoke<PingResponse>('ping');
}
