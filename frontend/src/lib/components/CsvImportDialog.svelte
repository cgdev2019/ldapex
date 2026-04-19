<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { formatError, ldapAdd, type Attribute, type AttributeValue } from '$lib/bridge';
  import Icon from './Icon.svelte';

  interface Props {
    parentDn?: string | null;
    onclose: () => void;
    oncreated?: () => void;
  }

  let { parentDn = null, onclose, oncreated }: Props = $props();

  let raw = $state(
    'uid,cn,sn,mail\nalice,Alice Wonder,Wonder,alice@example.org\nbob,Bob Smith,Smith,bob@example.org'
  );
  let dnTemplate = $state(parentDn ? `uid={uid},${parentDn}` : 'uid={uid},ou=People,dc=example,dc=org');
  let objectClasses = $state('top, inetOrgPerson, organizationalPerson, person');
  let working = $state(false);
  let progress = $state(0);
  let total = $state(0);
  let log = $state<string[]>([]);

  interface Row {
    raw: Record<string, string>;
    dn: string;
    attributes: Attribute[];
  }

  function parseCsv(text: string): { headers: string[]; rows: string[][] } {
    const lines = text.split(/\r?\n/).filter((l) => l.length > 0);
    if (lines.length === 0) return { headers: [], rows: [] };
    const split = (l: string) => splitCsv(l);
    return { headers: split(lines[0]), rows: lines.slice(1).map(split) };
  }

  /** Tiny RFC 4180-ish splitter — handles double-quoted fields with embedded commas. */
  function splitCsv(line: string): string[] {
    const out: string[] = [];
    let cur = '';
    let inQ = false;
    for (let i = 0; i < line.length; i++) {
      const c = line[i];
      if (inQ) {
        if (c === '"' && line[i + 1] === '"') {
          cur += '"';
          i++;
        } else if (c === '"') {
          inQ = false;
        } else cur += c;
      } else {
        if (c === ',') {
          out.push(cur);
          cur = '';
        } else if (c === '"') inQ = true;
        else cur += c;
      }
    }
    out.push(cur);
    return out.map((v) => v.trim());
  }

  const parsed = $derived(parseCsv(raw));

  const preview = $derived.by<Row[]>(() => {
    const ocList = objectClasses
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    return parsed.rows.map((cells) => {
      const record: Record<string, string> = {};
      parsed.headers.forEach((h, i) => (record[h] = cells[i] ?? ''));
      const dn = expand(dnTemplate, record);
      const attrs: Attribute[] = [];
      attrs.push({
        name: 'objectClass',
        values: ocList.map<AttributeValue>((c) => ({ kind: 'text', data: c }))
      });
      for (const [k, v] of Object.entries(record)) {
        if (!v.trim()) continue;
        attrs.push({
          name: k,
          values: [{ kind: 'text', data: v }]
        });
      }
      return { raw: record, dn, attributes: attrs };
    });
  });

  function expand(tpl: string, vars: Record<string, string>): string {
    return tpl.replace(/\{(\w+)\}/g, (_, k: string) => vars[k] ?? `{${k}}`);
  }

  async function apply() {
    if (preview.length === 0) return;
    working = true;
    log = [];
    progress = 0;
    total = preview.length;
    for (const row of preview) {
      try {
        await ldapAdd(row.dn, row.attributes);
        log = [...log, `✓ ${row.dn}`];
      } catch (err) {
        log = [...log, `✗ ${row.dn} — ${formatError(err)}`];
      } finally {
        progress += 1;
      }
    }
    working = false;
    oncreated?.();
  }

  function close() {
    if (!working) onclose();
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label="Import CSV → LDIF" onclick={close} tabindex="-1">
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header>
      <h2>
        <Icon name="import" size={16} />
        <span>Import CSV → LDAP</span>
      </h2>
      <button type="button" class="ghost icon-only" onclick={close} disabled={working}>
        <Icon name="x" size={14} />
      </button>
    </header>

    <div class="body">
      <label>
        <span>1. CSV (1ʳᵉ ligne = en-têtes)</span>
        <textarea rows="6" bind:value={raw} spellcheck="false"></textarea>
        {#if parsed.headers.length > 0}
          <p class="hint">
            <strong>{parsed.rows.length}</strong> ligne{parsed.rows.length > 1 ? 's' : ''}, colonnes :
            {#each parsed.headers as h, i (i)}
              <code class="inline">{h}</code>
            {/each}
          </p>
        {/if}
      </label>

      <label>
        <span>2. Template DN <span class="muted">— utilise <code>{'{colonne}'}</code></span></span>
        <input type="text" bind:value={dnTemplate} spellcheck="false" />
      </label>

      <label>
        <span>3. Object classes (séparées par des virgules)</span>
        <input type="text" bind:value={objectClasses} spellcheck="false" />
      </label>

      {#if preview.length > 0}
        <div class="preview">
          <h4>Aperçu (3 premières)</h4>
          <ul>
            {#each preview.slice(0, 3) as row, i (i)}
              <li>
                <code class="dn">{row.dn}</code>
                <ul class="attrs">
                  {#each row.attributes as a (a.name)}
                    <li>
                      <code>{a.name}</code>:
                      {a.values
                        .map((v) => (v.kind === 'text' ? v.data : '<bin>'))
                        .join(', ')}
                    </li>
                  {/each}
                </ul>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if working || progress > 0}
        <div class="progress">
          <div class="bar-bg">
            <div class="bar-fg" style:width={`${total ? (progress * 100) / total : 0}%`}></div>
          </div>
          <span>{progress}/{total}</span>
        </div>
        <pre class="log">{log.join('\n')}</pre>
      {/if}
    </div>

    <footer>
      <button type="button" class="ghost" onclick={close} disabled={working}>Fermer</button>
      <button
        type="button"
        class="primary"
        onclick={apply}
        disabled={working || preview.length === 0}
      >
        <Icon name="plus" size={14} />
        <span>{working ? 'Création…' : `Créer ${preview.length} entrée${preview.length > 1 ? 's' : ''}`}</span>
      </button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 55%, transparent);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    z-index: 200;
    padding: 1rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    width: min(92vw, 44rem);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.85rem 1.1rem;
    border-bottom: 1px solid var(--color-border);
  }

  header h2 {
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
  }

  .body {
    padding: 1rem 1.1rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: var(--text-sm);
  }

  label > span {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .muted {
    color: var(--color-text-subtle);
    font-weight: 400;
  }

  textarea,
  input {
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }

  .hint {
    font-size: 0.72rem;
    color: var(--color-text-subtle);
  }

  .inline {
    margin: 0 0.15rem;
    font-size: 0.7rem;
  }

  .preview {
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.75rem;
  }

  .preview h4 {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-text-subtle);
    font-weight: 600;
    margin-bottom: 0.4rem;
  }

  .preview > ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .dn {
    font-size: 0.78rem;
  }

  .attrs {
    list-style: none;
    margin: 0.2rem 0 0;
    padding: 0 0 0 0.85rem;
    border-left: 1px dashed var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .attrs li {
    word-break: break-all;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .bar-bg {
    flex: 1;
    height: 0.4rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-pill);
    overflow: hidden;
  }

  .bar-fg {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.2s ease;
  }

  .log {
    max-height: 8rem;
    overflow: auto;
    margin: 0;
    padding: 0.45rem 0.55rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-md);
    white-space: pre-wrap;
    word-break: break-all;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0.7rem 1.1rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }
</style>
