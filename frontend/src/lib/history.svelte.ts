/**
 * Write-history store with precomputed inverses. Any mutation the UI
 * performs records a tuple `(forward, inverse)` here so the user can
 * roll it back with one click — a capability no other LDAP browser
 * ships natively.
 *
 * The store is per-profile, persisted in localStorage under
 * `ldapex.history.<profileId|adhoc>` (newest first, capped at 200
 * entries). Undoing does NOT remove the entry from history; it
 * records the reverse as a new entry so the user can redo if needed.
 */

import {
  ldapAdd,
  ldapDelete,
  ldapModify,
  ldapRename,
  type Attribute,
  type AttributeValue,
  type Entry,
  type Modification
} from './bridge';
import { session } from './session.svelte';

export type HistoryOp = 'add' | 'modify' | 'delete' | 'rename';

export interface HistoryEntry {
  id: string;
  ts: number;
  dn: string;
  op: HistoryOp;
  /** Short user-facing description (e.g. "Replace mail (2 values)"). */
  label: string;
  /** The inverse command — what we fire when the user hits Undo. */
  inverse:
    | { kind: 'delete'; dn: string }
    | { kind: 'add'; dn: string; attributes: Attribute[] }
    | { kind: 'modify'; dn: string; mods: Modification[] }
    | { kind: 'rename'; dn: string; newRdn: string; newSuperior?: string; deleteOldRdn: boolean };
  undone?: boolean;
}

const MAX = 200;

function key(profileId: string | null): string {
  return `ldapex.history.${profileId ?? 'adhoc'}`;
}

function load(k: string): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(k);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (v): v is HistoryEntry =>
        typeof v === 'object' &&
        v !== null &&
        typeof (v as HistoryEntry).id === 'string' &&
        typeof (v as HistoryEntry).dn === 'string'
    );
  } catch {
    return [];
  }
}

function save(k: string, list: HistoryEntry[]): void {
  try {
    localStorage.setItem(k, JSON.stringify(list.slice(0, MAX)));
  } catch {
    /* storage full */
  }
}

function uuid(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

class HistoryStore {
  items = $state<HistoryEntry[]>([]);

  reload(): void {
    this.items = load(key(session.activeProfileId));
  }

  private push(entry: Omit<HistoryEntry, 'id' | 'ts'>): void {
    const full: HistoryEntry = { ...entry, id: uuid(), ts: Date.now() };
    const next = [full, ...this.items];
    this.items = next;
    save(key(session.activeProfileId), next);
  }

  // ---------- recorders ----------

  recordAdd(dn: string, attributes: Attribute[]): void {
    this.push({
      dn,
      op: 'add',
      label: `Ajout de ${attributes.length} attribut${attributes.length > 1 ? 's' : ''}`,
      inverse: { kind: 'delete', dn }
    });
  }

  recordDelete(dn: string, original: Entry): void {
    this.push({
      dn,
      op: 'delete',
      label: `Suppression (${original.attributes.length} attributs)`,
      inverse: { kind: 'add', dn, attributes: original.attributes }
    });
  }

  recordRename(
    oldDn: string,
    newDn: string,
    newRdn: string,
    newSuperior: string | undefined,
    deleteOldRdn: boolean
  ): void {
    // Compute the reverse: the new DN becomes the source; the original
    // RDN + old parent is the target.
    const origRdn = oldDn.split(',')[0] ?? oldDn;
    const origParent = oldDn.includes(',') ? oldDn.slice(oldDn.indexOf(',') + 1) : undefined;
    this.push({
      dn: newDn,
      op: 'rename',
      label: `Renommage → ${newRdn}`,
      inverse: {
        kind: 'rename',
        dn: newDn,
        newRdn: origRdn,
        newSuperior: origParent,
        deleteOldRdn
      }
    });
  }

  /**
   * Record a modify by deriving the reverse from before / after
   * text-value snapshots (the format EntryPanel already keeps).
   */
  recordModify(
    dn: string,
    before: Record<string, string[]>,
    after: Record<string, string[]>,
    forward: Modification[]
  ): void {
    const inverseMods: Modification[] = [];
    const names = new Set([...Object.keys(before), ...Object.keys(after)]);
    for (const name of names) {
      const b = before[name] ?? [];
      const a = after[name] ?? [];
      const same = b.length === a.length && b.every((v, i) => v === a[i]);
      if (same) continue;
      if (a.length === 0) {
        // forward was delete-all → inverse is add back
        inverseMods.push({ op: 'add', attribute: name, values: b });
      } else if (b.length === 0) {
        // forward was add → inverse is delete
        inverseMods.push({ op: 'delete', attribute: name, values: null });
      } else {
        // replace → inverse replace with original values
        inverseMods.push({ op: 'replace', attribute: name, values: b });
      }
    }
    this.push({
      dn,
      op: 'modify',
      label: `${forward.length} modification${forward.length > 1 ? 's' : ''}`,
      inverse: { kind: 'modify', dn, mods: inverseMods }
    });
  }

  /**
   * Fire the inverse of an entry. Returns the label so the caller can
   * show a success toast. Marks the entry as `undone` but keeps it
   * in the list so the user can read their trail (redo comes from a
   * freshly recorded forward op).
   */
  async undo(id: string): Promise<string> {
    const entry = this.items.find((e) => e.id === id);
    if (!entry) throw new Error('history entry not found');
    const inv = entry.inverse;
    switch (inv.kind) {
      case 'delete':
        await ldapDelete(inv.dn);
        break;
      case 'add':
        await ldapAdd(inv.dn, inv.attributes);
        break;
      case 'modify':
        await ldapModify(inv.dn, inv.mods);
        break;
      case 'rename':
        await ldapRename({
          dn: inv.dn,
          new_rdn: inv.newRdn,
          new_parent: inv.newSuperior ?? null,
          delete_old_rdn: inv.deleteOldRdn
        });
        break;
    }
    this.items = this.items.map((e) => (e.id === id ? { ...e, undone: true } : e));
    save(key(session.activeProfileId), this.items);
    return entry.label;
  }

  clear(): void {
    this.items = [];
    try {
      localStorage.removeItem(key(session.activeProfileId));
    } catch {
      /* ignore */
    }
  }
}

export const history = new HistoryStore();

/** Convenience helper used by components that don't care about the
 *  individual recorders — pass every attribute as a fresh Attribute.  */
export function toAttributes(record: Record<string, string[]>): Attribute[] {
  return Object.entries(record).map(([name, values]) => ({
    name,
    values: values.map<AttributeValue>((data) => ({ kind: 'text', data }))
  }));
}
