/**
 * Per-profile bookmark store backed by localStorage. Bookmarks are
 * pure UI sugar — the directory itself does not know about them.
 *
 * Storage key: `ldapex.bookmarks.<profileId|'adhoc'>` holding an
 * array of `{ dn, label, addedAt }` records (most recent first).
 */

import { session } from './session.svelte';

export interface Bookmark {
  dn: string;
  label: string;
  addedAt: string;
}

const MAX = 200;

function key(profileId: string | null): string {
  return `ldapex.bookmarks.${profileId ?? 'adhoc'}`;
}

function load(k: string): Bookmark[] {
  try {
    const raw = localStorage.getItem(k);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (v): v is Bookmark =>
        typeof v === 'object' &&
        v !== null &&
        typeof (v as Bookmark).dn === 'string' &&
        typeof (v as Bookmark).label === 'string'
    );
  } catch {
    return [];
  }
}

function persist(k: string, items: Bookmark[]): void {
  try {
    localStorage.setItem(k, JSON.stringify(items));
  } catch {
    /* quota full or disabled */
  }
}

class BookmarkStore {
  items = $state<Bookmark[]>([]);

  /** Pull the bookmark list for the currently-active profile. */
  reload(): void {
    this.items = load(key(session.activeProfileId));
  }

  has(dn: string): boolean {
    return this.items.some((b) => b.dn === dn);
  }

  add(dn: string, label?: string): void {
    const trimmedDn = dn.trim();
    if (!trimmedDn) return;
    const k = key(session.activeProfileId);
    const existing = this.items.filter((b) => b.dn !== trimmedDn);
    const next: Bookmark[] = [
      {
        dn: trimmedDn,
        label: (label ?? deriveLabel(trimmedDn)).trim() || trimmedDn,
        addedAt: new Date().toISOString()
      },
      ...existing
    ].slice(0, MAX);
    this.items = next;
    persist(k, next);
  }

  remove(dn: string): void {
    const k = key(session.activeProfileId);
    this.items = this.items.filter((b) => b.dn !== dn);
    persist(k, this.items);
  }

  toggle(dn: string, label?: string): void {
    if (this.has(dn)) this.remove(dn);
    else this.add(dn, label);
  }
}

function deriveLabel(dn: string): string {
  const first = dn.split(',')[0] ?? dn;
  const eq = first.indexOf('=');
  return eq > 0 ? first.slice(eq + 1) : first;
}

export const bookmarks = new BookmarkStore();

/**
 * Recent DN tracker — separate from bookmarks. Updated every time the
 * EntryPanel loads a DN so the command palette has a fast Cmd+K list.
 */
export interface Recent {
  dn: string;
  label: string;
  visitedAt: string;
}

const RECENT_MAX = 30;

function recentKey(profileId: string | null): string {
  return `ldapex.recent-dns.${profileId ?? 'adhoc'}`;
}

class RecentStore {
  items = $state<Recent[]>([]);

  reload(): void {
    try {
      const raw = localStorage.getItem(recentKey(session.activeProfileId));
      if (!raw) {
        this.items = [];
        return;
      }
      const parsed: unknown = JSON.parse(raw);
      this.items = Array.isArray(parsed)
        ? (parsed.filter(
            (v) =>
              typeof v === 'object' &&
              v !== null &&
              typeof (v as Recent).dn === 'string'
          ) as Recent[])
        : [];
    } catch {
      this.items = [];
    }
  }

  visit(dn: string): void {
    const label = deriveLabel(dn);
    const rest = this.items.filter((r) => r.dn !== dn);
    const next = [{ dn, label, visitedAt: new Date().toISOString() }, ...rest].slice(
      0,
      RECENT_MAX
    );
    this.items = next;
    try {
      localStorage.setItem(recentKey(session.activeProfileId), JSON.stringify(next));
    } catch {
      /* ignore */
    }
  }
}

export const recents = new RecentStore();
