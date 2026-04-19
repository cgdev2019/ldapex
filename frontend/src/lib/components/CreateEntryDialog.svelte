<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    ldapAdd,
    ldapFetchSchema,
    type Attribute,
    type ObjectClassDef,
    type SchemaInfo
  } from '$lib/bridge';
  import Icon from './Icon.svelte';

  interface Props {
    parentDn: string;
    onclose: () => void;
    oncreated?: (dn: string) => void;
  }

  let { parentDn, onclose, oncreated }: Props = $props();

  let schema = $state<SchemaInfo | null>(null);
  let loadingSchema = $state(true);
  let schemaError = $state<string | null>(null);

  let selectedClassNames = $state<string[]>(['inetOrgPerson']);
  let rdn = $state(get(_)('create_entry.default_rdn'));
  let attrValues = $state<Record<string, string>>({});
  let saving = $state(false);
  let error = $state<string | null>(null);

  // --- combobox state ------------------------------------------------
  let query = $state('');
  let comboOpen = $state(false);
  let highlighted = $state(0);
  let comboInputEl: HTMLInputElement | undefined = $state();
  let comboRootEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    loadSchema();
  });

  async function loadSchema() {
    loadingSchema = true;
    schemaError = null;
    try {
      schema = await ldapFetchSchema();
    } catch (err) {
      schemaError = formatError(err);
    } finally {
      loadingSchema = false;
    }
  }

  const selectedClasses = $derived<ObjectClassDef[]>(
    schema
      ? selectedClassNames
          .map((n) =>
            schema!.object_classes.find((c) => c.name.toLowerCase() === n.toLowerCase())
          )
          .filter((c): c is ObjectClassDef => c !== undefined)
      : []
  );

  /**
   * Suggestions = schema OCs that are not yet selected and match the
   * current query. Capped to 20 to keep the dropdown comfortable; real
   * schemas expose ~200 classes — the user types to narrow down.
   */
  const suggestions = $derived.by<ObjectClassDef[]>(() => {
    if (!schema) return [];
    const q = query.trim().toLowerCase();
    const available = schema.object_classes.filter(
      (c) => c.kind !== 'abstract' && !selectedClassNames.includes(c.name)
    );
    const matched = q ? available.filter((c) => c.name.toLowerCase().includes(q)) : available;
    // Sort: structural first, then alphabetical
    matched.sort((a, b) => {
      if (a.kind !== b.kind) return a.kind === 'structural' ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    return matched.slice(0, 20);
  });

  const must = $derived(uniqueAttrs(selectedClasses.flatMap((c) => c.must)));
  const may = $derived(uniqueAttrs(selectedClasses.flatMap((c) => c.may)));
  const dnPreview = $derived(rdn.trim() ? `${rdn.trim()},${parentDn}` : `…,${parentDn}`);

  function uniqueAttrs(arr: string[]): string[] {
    const out: string[] = [];
    const seen = new Set<string>();
    for (const a of arr) {
      const lower = a.toLowerCase();
      if (!seen.has(lower)) {
        seen.add(lower);
        out.push(a);
      }
    }
    return out;
  }

  function addClass(name: string) {
    if (!selectedClassNames.includes(name)) {
      selectedClassNames = [...selectedClassNames, name];
    }
    query = '';
    highlighted = 0;
    comboInputEl?.focus();
  }

  function removeClass(name: string) {
    selectedClassNames = selectedClassNames.filter((n) => n !== name);
  }

  function openCombo() {
    comboOpen = true;
    highlighted = 0;
  }

  function closeCombo() {
    comboOpen = false;
  }

  function onComboKey(event: KeyboardEvent) {
    if (!comboOpen) {
      if (event.key === 'ArrowDown' || event.key === 'Enter') {
        openCombo();
        event.preventDefault();
      }
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      highlighted = Math.min(highlighted + 1, suggestions.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const pick = suggestions[highlighted];
      if (pick) addClass(pick.name);
    } else if (event.key === 'Escape') {
      closeCombo();
    } else if (event.key === 'Backspace' && query === '' && selectedClassNames.length > 0) {
      selectedClassNames = selectedClassNames.slice(0, -1);
    }
  }

  function onDocPointer(event: PointerEvent) {
    if (!comboOpen) return;
    if (comboRootEl && !comboRootEl.contains(event.target as Node)) closeCombo();
  }

  $effect(() => {
    if (!comboOpen) return;
    document.addEventListener('pointerdown', onDocPointer, true);
    return () => document.removeEventListener('pointerdown', onDocPointer, true);
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    const dn = `${rdn.trim()},${parentDn}`;

    const attributes: Attribute[] = [];
    attributes.push({
      name: 'objectClass',
      values: selectedClassNames.map((n) => ({ kind: 'text', data: n }))
    });

    const rdnMatch = /^([^=]+)=(.+)$/.exec(rdn.trim());
    if (rdnMatch) {
      const [, rdnAttr, rdnValue] = rdnMatch;
      if (!attrValues[rdnAttr] || attrValues[rdnAttr].trim() === '') {
        attrValues[rdnAttr] = rdnValue;
      }
    }

    for (const [name, raw] of Object.entries(attrValues)) {
      if (raw.trim() === '') continue;
      attributes.push({
        name,
        values: raw
          .split('\n')
          .map((s) => s.trim())
          .filter((s) => s.length > 0)
          .map((s) => ({ kind: 'text', data: s }))
      });
    }

    saving = true;
    try {
      await ldapAdd(dn, attributes);
      const { history } = await import('$lib/history.svelte');
      history.recordAdd(dn, attributes);
      oncreated?.(dn);
      onclose();
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget && !saving) onclose();
  }
</script>

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-label={$_('create_entry.title')}
  onclick={handleBackdrop}
  onkeydown={(e) => e.key === 'Escape' && !saving && onclose()}
  tabindex="-1"
>
  <div class="dialog">
    <header>
      <div class="hd-text">
        <h2>
          <Icon name="plus" size={16} />
          <span>{$_('create_entry.title')}</span>
        </h2>
        <p class="parent">
          <Icon name="folder" size={11} />
          <code>{parentDn}</code>
        </p>
      </div>
      <button
        type="button"
        class="ghost icon-only"
        onclick={onclose}
        disabled={saving}
        aria-label={$_('common.close')}
      >
        <Icon name="x" size={16} />
      </button>
    </header>

    <form onsubmit={submit}>
      <div class="body">
        <!-- SECTION 1 : object classes -->
        <section class="step">
          <div class="step-head">
            <span class="step-num">1</span>
            <h3>{$_('create_entry.object_classes_label')}</h3>
            {#if selectedClassNames.length > 0}
              <span class="count">{selectedClassNames.length}</span>
            {/if}
          </div>

          {#if loadingSchema}
            <p class="status"><Icon name="refresh" size={13} /> {$_('create_entry.fetching_schema')}</p>
          {:else if schemaError}
            <p class="status error">{$_('create_entry.schema_error', { values: { message: schemaError } })}</p>
          {:else if schema}
            <div class="combobox" bind:this={comboRootEl}>
              <div class="combo-field" class:focused={comboOpen}>
                {#each selectedClasses as oc (oc.name)}
                  <span class="chip" data-kind={oc.kind}>
                    <span>{oc.name}</span>
                    <button
                      type="button"
                      class="chip-x"
                      onclick={() => removeClass(oc.name)}
                      aria-label="Remove {oc.name}"
                    >
                      <Icon name="x" size={10} />
                    </button>
                  </span>
                {/each}
                <input
                  type="text"
                  bind:this={comboInputEl}
                  bind:value={query}
                  placeholder={selectedClassNames.length === 0
                    ? 'inetOrgPerson, organizationalUnit, …'
                    : ''}
                  onfocus={openCombo}
                  oninput={openCombo}
                  onkeydown={onComboKey}
                  spellcheck="false"
                  autocomplete="off"
                />
                <Icon name="chevron-down" size={13} class="combo-caret" />
              </div>

              {#if comboOpen && suggestions.length > 0}
                <div class="dropdown" role="listbox">
                  {#each suggestions as oc, i (oc.name)}
                    <button
                      type="button"
                      role="option"
                      aria-selected={i === highlighted}
                      class="option"
                      class:highlighted={i === highlighted}
                      onclick={() => addClass(oc.name)}
                      onmouseenter={() => (highlighted = i)}
                    >
                      <span class="o-name">{oc.name}</span>
                      <span class="o-kind" data-kind={oc.kind}>{oc.kind}</span>
                    </button>
                  {/each}
                </div>
              {:else if comboOpen && suggestions.length === 0}
                <div class="dropdown empty">No match</div>
              {/if}
            </div>

            <p class="step-hint">
              <Icon name="info" size={11} />
              <span>
                Start typing to find an objectClass. Use <kbd>↑</kbd><kbd>↓</kbd><kbd>Enter</kbd> to pick.
              </span>
            </p>
          {/if}
        </section>

        <!-- SECTION 2 : RDN -->
        <section class="step">
          <div class="step-head">
            <span class="step-num">2</span>
            <h3>{$_('create_entry.rdn')}</h3>
            <span class="req">required</span>
          </div>

          <input
            type="text"
            bind:value={rdn}
            required
            spellcheck="false"
            class="rdn-input"
            placeholder="cn=John Doe"
          />
          <p class="dn-preview">
            <span class="dim">DN</span>
            <code>{dnPreview}</code>
          </p>
        </section>

        <!-- SECTION 3 : MUST -->
        {#if must.length > 0}
          <section class="step">
            <div class="step-head">
              <span class="step-num">3</span>
              <h3>{$_('create_entry.must')}</h3>
              <span class="count">{must.length}</span>
            </div>

            <div class="rows">
              {#each must as name (name)}
                {@const current = attrValues[name] ?? ''}
                <label class="kv">
                  <span class="lab"><code>{name}</code><span class="req-star">*</span></span>
                  <textarea
                    rows="1"
                    oninput={(e) =>
                      (attrValues = {
                        ...attrValues,
                        [name]: (e.currentTarget as HTMLTextAreaElement).value
                      })}
                    value={current}
                  ></textarea>
                </label>
              {/each}
            </div>
          </section>
        {/if}

        <!-- SECTION 4 : MAY -->
        {#if may.length > 0}
          <section class="step">
            <div class="step-head">
              <span class="step-num">{must.length > 0 ? 4 : 3}</span>
              <h3>{$_('create_entry.may')}</h3>
              <span class="count muted">{may.length}</span>
            </div>

            <div class="rows">
              {#each may.slice(0, 15) as name (name)}
                {@const current = attrValues[name] ?? ''}
                <label class="kv">
                  <span class="lab"><code>{name}</code></span>
                  <textarea
                    rows="1"
                    oninput={(e) =>
                      (attrValues = {
                        ...attrValues,
                        [name]: (e.currentTarget as HTMLTextAreaElement).value
                      })}
                    value={current}
                  ></textarea>
                </label>
              {/each}
              {#if may.length > 15}
                <p class="hint">
                  {$_('create_entry.may_hidden', { values: { count: may.length - 15 } })}
                </p>
              {/if}
            </div>
          </section>
        {/if}
      </div>

      {#if error}
        <p class="status banner error">{error}</p>
      {/if}

      <footer class="actions">
        <button type="button" class="ghost" onclick={onclose} disabled={saving}>
          {$_('common.cancel')}
        </button>
        <button type="submit" class="primary" disabled={saving || selectedClassNames.length === 0}>
          <Icon name={saving ? 'refresh' : 'plus'} size={14} />
          <span>{saving ? $_('create_entry.submitting') : $_('create_entry.submit')}</span>
        </button>
      </footer>
    </form>
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
    padding: 1.25rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    width: min(92vw, 40rem);
    max-height: min(92vh, 44rem);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1rem 1.3rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .hd-text {
    flex: 1;
    min-width: 0;
  }

  .hd-text h2 {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.1rem;
  }

  .parent {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    margin-top: 0.15rem;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .parent code {
    padding: 0.05em 0.35em;
    font-size: 0.7rem;
    max-width: 30rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  form {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 1rem 1.3rem 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .step {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .step-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 1.6rem;
  }

  .step-num {
    display: grid;
    place-items: center;
    width: 1.35rem;
    height: 1.35rem;
    border-radius: 50%;
    background: var(--color-primary-soft);
    color: var(--color-primary);
    font-weight: 600;
    font-size: 0.7rem;
    flex-shrink: 0;
  }

  .step-head h3 {
    flex: 1;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text);
  }

  .count {
    font-size: 0.68rem;
    padding: 0.05rem 0.5rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
    color: white;
    font-weight: 600;
  }

  .count.muted {
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
  }

  .req {
    font-size: 0.65rem;
    color: var(--color-danger);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .step-hint {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.72rem;
    color: var(--color-text-subtle);
  }

  .step-hint kbd {
    margin: 0 0.15rem;
  }

  /* -------- combobox -------- */

  .combobox {
    position: relative;
  }

  .combo-field {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    padding: 0.35rem 0.45rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    min-height: 2.35rem;
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .combo-field.focused,
  .combo-field:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .combo-field input {
    flex: 1;
    min-width: 6rem;
    border: none;
    padding: 0.2rem 0.3rem;
    background: transparent;
    box-shadow: none;
  }

  .combo-field input:focus {
    border: none;
    box-shadow: none;
    outline: none;
  }

  :global(svg.combo-caret) {
    color: var(--color-text-subtle);
    margin-right: 0.2rem;
    flex-shrink: 0;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.15rem 0.35rem 0.15rem 0.55rem;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: 500;
    background: var(--color-primary-soft);
    color: var(--color-primary);
    border: 1px solid color-mix(in oklab, var(--color-primary) 25%, transparent);
  }

  .chip[data-kind='auxiliary'] {
    background: color-mix(in oklab, var(--color-warning) 18%, transparent);
    color: var(--color-warning);
    border-color: color-mix(in oklab, var(--color-warning) 35%, transparent);
  }

  .chip-x {
    display: inline-grid;
    place-items: center;
    width: 1rem;
    height: 1rem;
    padding: 0;
    margin-left: 0.1rem;
    border-radius: 50%;
    border: none;
    background: transparent;
    color: inherit;
    opacity: 0.7;
  }

  .chip-x:hover {
    background: color-mix(in oklab, currentColor 20%, transparent);
    opacity: 1;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 0.35rem);
    left: 0;
    right: 0;
    z-index: 10;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    max-height: 16rem;
    overflow: auto;
    padding: 0.25rem;
  }

  .dropdown.empty {
    padding: 0.8rem;
    text-align: center;
    color: var(--color-text-subtle);
    font-size: var(--text-sm);
    font-style: italic;
  }

  .option {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.55rem;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    font-size: var(--text-sm);
    color: var(--color-text);
  }

  .option:hover,
  .option.highlighted {
    background: var(--color-surface-hover);
  }

  .o-name {
    font-weight: 500;
  }

  .o-kind {
    font-size: 0.6rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 0.05rem 0.5rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
    color: var(--color-text-subtle);
    font-weight: 600;
  }

  .o-kind[data-kind='structural'] {
    background: color-mix(in oklab, var(--color-primary) 18%, transparent);
    color: var(--color-primary);
  }

  .o-kind[data-kind='auxiliary'] {
    background: color-mix(in oklab, var(--color-warning) 18%, transparent);
    color: var(--color-warning);
  }

  /* -------- RDN -------- */

  .rdn-input {
    font-family: var(--font-mono);
    font-weight: 500;
  }

  .dn-preview {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .dn-preview code {
    font-size: 0.7rem;
    word-break: break-all;
  }

  .dim {
    color: var(--color-text-subtle);
    text-transform: uppercase;
    font-size: 0.63rem;
    letter-spacing: 0.08em;
    margin-right: 0.35rem;
    font-weight: 600;
  }

  /* -------- MUST / MAY -------- */

  .rows {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding-left: 1.85rem;
  }

  .kv {
    display: grid;
    grid-template-columns: 8.5rem 1fr;
    gap: 0.55rem;
    align-items: center;
  }

  .lab {
    display: inline-flex;
    align-items: center;
    gap: 0.15rem;
    font-size: var(--text-xs);
  }

  .req-star {
    color: var(--color-danger);
    font-weight: 700;
  }

  .kv textarea {
    min-height: 2rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    padding: 0.3rem 0.55rem;
    resize: vertical;
  }

  .hint {
    color: var(--color-text-subtle);
    font-size: var(--text-xs);
    font-style: italic;
    padding-left: 0.5rem;
  }

  /* -------- status / actions -------- */

  .status {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .status.error {
    color: var(--color-danger);
  }

  .status.banner {
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
    padding: 0.55rem 0.9rem;
    margin: 0 1.3rem;
    border-radius: var(--radius-md);
  }

  footer.actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    padding: 0.8rem 1.3rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }
</style>
