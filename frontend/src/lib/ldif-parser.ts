/**
 * RFC 2849 LDIF parser. The goal is to power the LDIF workbench, so
 * we accept every change-record shape Ldapex can execute:
 *
 *   add:     `dn: …` + optional `changetype: add` + attribute lines
 *   modify:  `dn: …` + `changetype: modify` + `{add|replace|delete}:
 *            attr` blocks separated by `-` terminator lines
 *   delete:  `dn: …` + `changetype: delete` (no body)
 *   modrdn:  `dn: …` + `changetype: modrdn|moddn` + `newrdn: …` +
 *            `deleteoldrdn: 0|1` + optional `newsuperior: …`
 *
 * Line folding (RFC 2849 §8 rule 2) — a leading space on a
 * continuation line — is honoured. `attr:: base64` values are decoded
 * to UTF-8 when possible, otherwise kept as the raw base64 string and
 * flagged binary (the backend's `ldap_add` ignores binary values in
 * MVP, so we do not over-promise).
 *
 * We deliberately ignore `attr:<url` (file: / http: values) and
 * `control: …` prefixes — both are rare and map poorly to our
 * current Tauri commands.
 */

import type { Attribute, Modification } from './bridge';

export type ChangeRecord =
  | { kind: 'add'; dn: string; attributes: Attribute[] }
  | { kind: 'delete'; dn: string }
  | { kind: 'modify'; dn: string; mods: Modification[] }
  | { kind: 'modrdn'; dn: string; newRdn: string; deleteOldRdn: boolean; newSuperior?: string };

export interface ParseResult {
  records: ChangeRecord[];
  errors: string[];
}

export function parseLdif(text: string): ParseResult {
  const errors: string[] = [];
  const records: ChangeRecord[] = [];

  // 1. Unfold continuation lines (a space-prefixed line is a suffix of
  //    the previous one).
  const rawLines = text.split(/\r?\n/);
  const lines: string[] = [];
  for (const l of rawLines) {
    if (l.startsWith(' ') && lines.length > 0) {
      lines[lines.length - 1] += l.slice(1);
    } else {
      lines.push(l);
    }
  }

  // 2. Split into records on blank lines. Discard comment lines (#…)
  //    and the optional `version:` header.
  let current: string[] = [];
  const blocks: string[][] = [];
  for (const l of lines) {
    if (l === '') {
      if (current.length > 0) {
        blocks.push(current);
        current = [];
      }
      continue;
    }
    if (l.startsWith('#')) continue;
    if (/^version\s*:/i.test(l) && current.length === 0) continue;
    current.push(l);
  }
  if (current.length > 0) blocks.push(current);

  // 3. Parse each block.
  for (const [blockIdx, block] of blocks.entries()) {
    try {
      const entries = block.map(parseLine);
      if (entries[0]?.[0].toLowerCase() !== 'dn') {
        throw new Error('block must start with `dn:`');
      }
      const dn = entries[0][1];
      const changeTypeEntry = entries.find(([a]) => a.toLowerCase() === 'changetype');
      const ct = changeTypeEntry?.[1]?.trim().toLowerCase() ?? 'add';

      switch (ct) {
        case 'add':
          records.push(buildAdd(dn, entries));
          break;
        case 'delete':
          records.push({ kind: 'delete', dn });
          break;
        case 'modify':
          records.push(buildModify(dn, entries));
          break;
        case 'modrdn':
        case 'moddn':
          records.push(buildModRdn(dn, entries));
          break;
        default:
          throw new Error(`unknown changetype "${ct}"`);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errors.push(`Record #${blockIdx + 1}: ${msg}`);
    }
  }

  return { records, errors };
}

/** `attr: value` / `attr:: base64` — returns `[name, value]`. */
function parseLine(line: string): [string, string] {
  const idx = line.indexOf(':');
  if (idx < 0) throw new Error(`missing ':' on line "${line}"`);
  const attr = line.slice(0, idx);
  let rest = line.slice(idx + 1);
  if (rest.startsWith(':')) {
    // base64-encoded
    const b64 = rest.slice(1).trim();
    try {
      const decoded = atob(b64);
      // Try UTF-8 decode — if it throws, keep the raw bytes as JS
      // string (ASCII subset is safe).
      try {
        return [attr, new TextDecoder('utf-8', { fatal: true }).decode(bytesFromBinary(decoded))];
      } catch {
        return [attr, decoded];
      }
    } catch {
      throw new Error(`invalid base64 for attribute ${attr}`);
    }
  }
  if (rest.startsWith('<')) {
    throw new Error(`URL-referenced values are not supported (attribute ${attr})`);
  }
  // Strip the single leading space the spec allows after the colon.
  if (rest.startsWith(' ')) rest = rest.slice(1);
  return [attr, rest];
}

function bytesFromBinary(s: string): Uint8Array {
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

function buildAdd(dn: string, entries: [string, string][]): ChangeRecord {
  const attrs: Attribute[] = [];
  const byName = new Map<string, string[]>();
  for (const [name, value] of entries) {
    const lower = name.toLowerCase();
    if (lower === 'dn' || lower === 'changetype') continue;
    const bucket = byName.get(name) ?? [];
    bucket.push(value);
    byName.set(name, bucket);
  }
  for (const [name, values] of byName.entries()) {
    attrs.push({
      name,
      values: values.map((v) => ({ kind: 'text', data: v }))
    });
  }
  return { kind: 'add', dn, attributes: attrs };
}

function buildModify(dn: string, entries: [string, string][]): ChangeRecord {
  // Skip dn + changetype, then consume blocks of:
  //   <op>: <attr>
  //   <attr>: <value>...
  //   -
  const body = entries.filter(([a]) => a.toLowerCase() !== 'dn' && a.toLowerCase() !== 'changetype');
  const mods: Modification[] = [];
  let i = 0;
  while (i < body.length) {
    const [opName, opTarget] = body[i];
    const op = opName.toLowerCase();
    if (op !== 'add' && op !== 'replace' && op !== 'delete') {
      throw new Error(`modify: unexpected "${opName}:" — expected add/replace/delete`);
    }
    const attr = opTarget.trim();
    const values: string[] = [];
    i++;
    while (i < body.length && body[i][0] !== '-') {
      const [name, value] = body[i];
      if (name.toLowerCase() !== attr.toLowerCase()) {
        throw new Error(
          `modify: attribute mismatch — expected ${attr}, got ${name}`
        );
      }
      values.push(value);
      i++;
    }
    // Consume the `-` terminator if present.
    if (i < body.length && body[i][0] === '-') i++;

    switch (op) {
      case 'add':
        mods.push({ op: 'add', attribute: attr, values });
        break;
      case 'replace':
        mods.push({ op: 'replace', attribute: attr, values });
        break;
      case 'delete':
        mods.push({
          op: 'delete',
          attribute: attr,
          values: values.length > 0 ? values : null
        });
        break;
    }
  }
  return { kind: 'modify', dn, mods };
}

function buildModRdn(dn: string, entries: [string, string][]): ChangeRecord {
  const byName: Record<string, string> = {};
  for (const [name, value] of entries) {
    byName[name.toLowerCase()] = value;
  }
  const newRdn = byName.newrdn;
  if (!newRdn) throw new Error('modrdn: missing `newrdn:`');
  const deleteOldRdn = byName.deleteoldrdn !== '0';
  const newSuperior = byName.newsuperior;
  return {
    kind: 'modrdn',
    dn,
    newRdn,
    deleteOldRdn,
    newSuperior: newSuperior ? newSuperior : undefined
  };
}

/** Human label for the change op — used in the preview. */
export function recordLabel(r: ChangeRecord): { op: string; color: 'add' | 'modify' | 'delete' | 'rename' } {
  switch (r.kind) {
    case 'add':
      return { op: `add (${r.attributes.length} attributs)`, color: 'add' };
    case 'modify':
      return { op: `modify (${r.mods.length} changement${r.mods.length > 1 ? 's' : ''})`, color: 'modify' };
    case 'delete':
      return { op: 'delete', color: 'delete' };
    case 'modrdn':
      return { op: `rename → ${r.newRdn}`, color: 'rename' };
  }
}
