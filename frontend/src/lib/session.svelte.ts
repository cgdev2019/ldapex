import { ldapConnect, ldapDisconnect, type ConnectInput } from './bridge';

/**
 * Global session state. Using Svelte 5 runes for reactivity. The Tauri
 * backend holds the real LDAP handle; this store tracks what the UI
 * needs to know (are we connected, what was the base DN used).
 */
class SessionStore {
  connected = $state(false);
  baseDn = $state<string | null>(null);
  bindDn = $state<string | null>(null);
  url = $state<string | null>(null);
  connecting = $state(false);

  async connect(input: ConnectInput, baseDn: string): Promise<void> {
    this.connecting = true;
    try {
      await ldapConnect(input);
      this.connected = true;
      this.baseDn = baseDn;
      this.bindDn = input.bind_dn;
      this.url = input.url;
    } finally {
      this.connecting = false;
    }
  }

  async disconnect(): Promise<void> {
    await ldapDisconnect();
    this.connected = false;
    this.baseDn = null;
    this.bindDn = null;
    this.url = null;
  }
}

export const session = new SessionStore();
