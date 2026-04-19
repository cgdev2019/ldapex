/**
 * Multi-selection store. Tracks the set of DNs the user has marked
 * (via Ctrl/Cmd+click in the tree or the search results) so a single
 * batch operation can apply to all of them. The set is purely
 * in-memory — selections do not survive a session restart on
 * purpose, since they're highly contextual.
 */

class SelectionStore {
  dns = $state<Set<string>>(new Set());
  /** Reactive count derived from the set. */
  count = $derived(this.dns.size);

  has(dn: string): boolean {
    return this.dns.has(dn);
  }

  toggle(dn: string): void {
    const next = new Set(this.dns);
    if (next.has(dn)) next.delete(dn);
    else next.add(dn);
    this.dns = next;
  }

  add(dn: string): void {
    if (this.dns.has(dn)) return;
    const next = new Set(this.dns);
    next.add(dn);
    this.dns = next;
  }

  remove(dn: string): void {
    if (!this.dns.has(dn)) return;
    const next = new Set(this.dns);
    next.delete(dn);
    this.dns = next;
  }

  clear(): void {
    this.dns = new Set();
  }

  list(): string[] {
    return Array.from(this.dns);
  }
}

export const selection = new SelectionStore();
