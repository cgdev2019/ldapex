/**
 * Per-DN user notes. Stored locally in localStorage under
 * `ldapex.notes.<profileId|adhoc>.<dn>` so a returning user can
 * annotate a service account ("monitored by Ops / rotate key
 * quarterly") without touching the directory itself.
 *
 * Keeping this side-car in localStorage is a first pass: a later
 * iteration can flush the store to a flat file in `~/.ldapex/notes/`
 * via the Tauri fs plugin so the annotations survive a browser cache
 * wipe — the API below stays stable.
 */

import { session } from './session.svelte';

function key(profileId: string | null, dn: string): string {
  return `ldapex.notes.${profileId ?? 'adhoc'}.${dn}`;
}

function indexKey(profileId: string | null): string {
  return `ldapex.notes-index.${profileId ?? 'adhoc'}`;
}

function loadIndex(profileId: string | null): Set<string> {
  try {
    const raw = localStorage.getItem(indexKey(profileId));
    if (!raw) return new Set();
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? new Set(parsed.filter((x) => typeof x === 'string')) : new Set();
  } catch {
    return new Set();
  }
}

function saveIndex(profileId: string | null, set: Set<string>) {
  try {
    localStorage.setItem(indexKey(profileId), JSON.stringify(Array.from(set)));
  } catch {
    /* storage full / disabled */
  }
}

class NotesStore {
  /** DNs that currently have a note under the active profile. */
  index = $state<Set<string>>(new Set());

  reload(): void {
    this.index = loadIndex(session.activeProfileId);
  }

  has(dn: string): boolean {
    return this.index.has(dn);
  }

  load(dn: string): string {
    try {
      return localStorage.getItem(key(session.activeProfileId, dn)) ?? '';
    } catch {
      return '';
    }
  }

  save(dn: string, text: string): void {
    const k = key(session.activeProfileId, dn);
    try {
      if (text.trim().length === 0) {
        localStorage.removeItem(k);
        const next = new Set(this.index);
        next.delete(dn);
        this.index = next;
        saveIndex(session.activeProfileId, next);
      } else {
        localStorage.setItem(k, text);
        if (!this.index.has(dn)) {
          const next = new Set(this.index);
          next.add(dn);
          this.index = next;
          saveIndex(session.activeProfileId, next);
        }
      }
    } catch {
      /* ignore */
    }
  }
}

export const notes = new NotesStore();
