<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    ldapDelete,
    ldapExportLdif,
    ldapModify,
    ldapReadEntry,
    type AttributeValue,
    type Entry,
    type Modification
  } from '$lib/bridge';
  import { bookmarks, recents } from '$lib/bookmarks.svelte';
  import { copyToClipboard } from '$lib/clipboard';
  import { history } from '$lib/history.svelte';
  import { notes } from '$lib/notes.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    dn: string | null;
    onentrychanged?: (event: { dn: string; kind: 'modified' | 'deleted' }) => void;
  }

  let { dn, onentrychanged }: Props = $props();

  let entry = $state<Entry | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let filter = $state('');

  let editing = $state(false);
  let draftText = $state<Record<string, string[]>>({});
  let originalText = $state<Record<string, string[]>>({});
  let newAttrName = $state('');
  let saving = $state(false);

  $effect(() => {
    if (dn) {
      load(dn);
    } else {
      resetAll();
    }
  });

  $effect(() => {
    const onSave = () => {
      if (editing && !saving) void save();
    };
    const onDelete = () => {
      if (dn && !editing && !saving) void confirmDelete();
    };
    window.addEventListener('ldapex:save', onSave);
    window.addEventListener('ldapex:delete', onDelete);
    return () => {
      window.removeEventListener('ldapex:save', onSave);
      window.removeEventListener('ldapex:delete', onDelete);
    };
  });

  function resetAll() {
    entry = null;
    error = null;
    editing = false;
    draftText = {};
    originalText = {};
    newAttrName = '';
  }

  async function load(target: string) {
    loading = true;
    error = null;
    editing = false;
    try {
      entry = await ldapReadEntry(target);
      originalText = textMapOf(entry);
      draftText = cloneMap(originalText);
      recents.visit(target);
    } catch (err) {
      error = formatError(err);
      entry = null;
    } finally {
      loading = false;
    }
  }

  const isBookmarked = $derived(dn ? bookmarks.has(dn) : false);

  function toggleBookmark() {
    if (!dn) return;
    bookmarks.toggle(dn);
  }

  let exporting = $state(false);
  let noteOpen = $state(false);
  let noteDraft = $state('');
  let noteTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (dn) {
      notes.reload();
      noteDraft = notes.load(dn);
      noteOpen = noteDraft.length > 0;
    } else {
      noteDraft = '';
      noteOpen = false;
    }
  });

  function queueSaveNote() {
    if (!dn) return;
    if (noteTimer) clearTimeout(noteTimer);
    const target = dn;
    const text = noteDraft;
    noteTimer = setTimeout(() => {
      notes.save(target, text);
    }, 400);
  }

  async function exportLdif() {
    if (!dn) return;
    exporting = true;
    error = null;
    try {
      const res = await ldapExportLdif({ base_dn: dn, scope: 'subtree' });
      await copyToClipboard(res.ldif);
      window.alert(
        get(_)('ldif.exported_entry', { values: { count: res.entry_count } })
      );
    } catch (err) {
      error = formatError(err);
    } finally {
      exporting = false;
    }
  }

  function textMapOf(e: Entry): Record<string, string[]> {
    const out: Record<string, string[]> = {};
    for (const attr of e.attributes) {
      const values = attr.values
        .filter((v): v is { kind: 'text'; data: string } => v.kind === 'text')
        .map((v) => v.data);
      if (values.length > 0) out[attr.name] = values;
    }
    return out;
  }

  function cloneMap(m: Record<string, string[]>): Record<string, string[]> {
    const out: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(m)) out[k] = [...v];
    return out;
  }

  function computeDiff(): Modification[] {
    const mods: Modification[] = [];
    const allNames = new Set([...Object.keys(originalText), ...Object.keys(draftText)]);
    for (const name of allNames) {
      const before = originalText[name] ?? [];
      const after = (draftText[name] ?? []).filter((v) => v.length > 0);
      const same =
        before.length === after.length && before.every((v, i) => v === after[i]);
      if (same) continue;
      if (after.length === 0) {
        mods.push({ op: 'delete', attribute: name, values: null });
      } else if (before.length === 0) {
        mods.push({ op: 'add', attribute: name, values: after });
      } else {
        mods.push({ op: 'replace', attribute: name, values: after });
      }
    }
    return mods;
  }

  const pendingMods = $derived(editing ? computeDiff() : []);

  async function save() {
    if (!dn) return;
    const mods = computeDiff();
    if (mods.length === 0) {
      editing = false;
      return;
    }
    saving = true;
    error = null;
    const before = cloneMap(originalText);
    const after = cloneMap(draftText);
    try {
      await ldapModify(dn, mods);
      history.recordModify(dn, before, after, mods);
      onentrychanged?.({ dn, kind: 'modified' });
      await load(dn);
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }

  function cancelEdit() {
    draftText = cloneMap(originalText);
    newAttrName = '';
    editing = false;
    error = null;
  }

  function updateValue(name: string, index: number, value: string) {
    const next = [...(draftText[name] ?? [])];
    next[index] = value;
    draftText = { ...draftText, [name]: next };
  }

  function addValue(name: string) {
    const next = [...(draftText[name] ?? []), ''];
    draftText = { ...draftText, [name]: next };
  }

  function removeValue(name: string, index: number) {
    const next = [...(draftText[name] ?? [])];
    next.splice(index, 1);
    draftText = { ...draftText, [name]: next };
  }

  function addAttribute() {
    const name = newAttrName.trim();
    if (!name || name in draftText) return;
    draftText = { ...draftText, [name]: [''] };
    newAttrName = '';
  }

  async function confirmDelete() {
    if (!dn) return;
    const yes = window.confirm(get(_)('entry.confirm_delete', { values: { dn } }));
    if (!yes) return;
    saving = true;
    error = null;
    const snapshot = entry; // capture before the write wipes state
    try {
      await ldapDelete(dn);
      if (snapshot) history.recordDelete(dn, snapshot);
      onentrychanged?.({ dn, kind: 'deleted' });
      resetAll();
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }

  async function copy(text: string) {
    try {
      await copyToClipboard(text);
    } catch {
      /* ignore */
    }
  }

  function isBinary(v: AttributeValue): v is { kind: 'binary'; data: string } {
    return v.kind === 'binary';
  }

  const displayAttributes = $derived(
    entry && filter
      ? entry.attributes.filter((a) => a.name.toLowerCase().includes(filter.toLowerCase()))
      : (entry?.attributes ?? [])
  );
</script>

<section class="panel">
  {#if !dn}
    <div class="empty">
      <div class="empty-icon"><Icon name="database" size={28} /></div>
      <p>{$_('entry.empty')}</p>
    </div>
  {:else if loading}
    <div class="empty">
      <Icon name="refresh" size={14} />
      <p>{$_('common.loading')}</p>
    </div>
  {:else if !entry}
    {#if error}
      <p class="status error">{error}</p>
    {/if}
  {:else}
    <header>
      <div class="title">
        <button
          type="button"
          class="ghost icon-only star"
          class:on={isBookmarked}
          onclick={toggleBookmark}
          aria-label={isBookmarked ? $_('bookmark.remove') : $_('bookmark.add')}
          title={isBookmarked ? $_('bookmark.remove') : $_('bookmark.add')}
        >
          <Icon name={isBookmarked ? 'star-filled' : 'star'} size={14} />
        </button>
        <Icon name="file-lock" size={14} />
        <code class="dn" title={entry.dn}>{entry.dn}</code>
      </div>
      <div class="actions">
        {#if editing}
          {#if pendingMods.length > 0}
            <span class="pill dirty" aria-live="polite">
              {pendingMods.length}
              {pendingMods.length === 1 ? 'change' : 'changes'}
            </span>
          {/if}
          <button type="button" class="ghost" onclick={cancelEdit} disabled={saving}>
            <Icon name="x" size={14} />
            <span>{$_('entry.actions.cancel')}</span>
          </button>
          <button type="button" class="primary" onclick={save} disabled={saving}>
            <Icon name={saving ? 'refresh' : 'save'} size={14} />
            <span>{saving ? $_('entry.actions.saving') : $_('entry.actions.save')}</span>
          </button>
        {:else}
          <label class="search-wrap">
            <Icon name="search" size={13} />
            <input type="search" placeholder={$_('entry.filter_placeholder')} bind:value={filter} />
          </label>
          <button
            type="button"
            class="ghost"
            onclick={exportLdif}
            disabled={exporting}
            title={$_('ldif.export_tooltip')}
          >
            <Icon name={exporting ? 'refresh' : 'export'} size={13} />
            <span>{exporting ? $_('ldif.exporting') : $_('ldif.export')}</span>
          </button>
          <button type="button" onclick={() => (editing = true)}>
            <Icon name="pencil" size={13} />
            <span>{$_('entry.actions.edit')}</span>
          </button>
          <button type="button" class="danger" onclick={confirmDelete} disabled={saving}>
            <Icon name="trash" size={13} />
            <span>{$_('entry.actions.delete')}</span>
          </button>
        {/if}
      </div>
    </header>

    {#if error}
      <p class="status banner error">{error}</p>
    {/if}

    <div class="body">
      {#if editing}
        <div class="edit-area">
          {#each Object.entries(draftText) as [name, values] (name)}
            <div class="edit-field">
              <div class="edit-head">
                <code class="attr-name">{name}</code>
                <span class="value-count">{values.length}</span>
              </div>
              <div class="edit-values">
                {#each values as _v, i}
                  <div class="edit-row">
                    <input
                      type="text"
                      value={values[i]}
                      oninput={(e) =>
                        updateValue(name, i, (e.currentTarget as HTMLInputElement).value)}
                    />
                    <button
                      type="button"
                      class="ghost icon-only"
                      onclick={() => removeValue(name, i)}
                      aria-label="Remove value"
                    >
                      <Icon name="x" size={13} />
                    </button>
                  </div>
                {/each}
                <button type="button" class="ghost sm add-val" onclick={() => addValue(name)}>
                  <Icon name="plus" size={12} />
                  <span>{$_('entry.add_value')}</span>
                </button>
              </div>
            </div>
          {/each}

          <div class="add-attr">
            <input
              type="text"
              placeholder={$_('entry.new_attribute_placeholder')}
              bind:value={newAttrName}
              onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addAttribute())}
            />
            <button type="button" class="primary sm" onclick={addAttribute}>
              <Icon name="plus" size={13} />
              <span>{$_('entry.add_attribute')}</span>
            </button>
          </div>
        </div>
      {:else}
        <table>
          <thead>
            <tr>
              <th scope="col">{$_('entry.column_attribute')}</th>
              <th scope="col">{$_('entry.column_value')}</th>
            </tr>
          </thead>
          <tbody>
            {#each displayAttributes as attr (attr.name)}
              {@const values = attr.values}
              {#each values as value, i (i)}
                <tr>
                  {#if i === 0}
                    <th scope="row" rowspan={values.length}>
                      <code>{attr.name}</code>
                    </th>
                  {/if}
                  <td>
                    {#if isBinary(value)}
                      <span class="binary" title="{value.data.length} b64 chars">
                        <Icon name="file-lock" size={11} />
                        <span>
                          {$_('entry.binary_value', { values: { count: value.data.length } })}
                        </span>
                      </span>
                    {:else}
                      <button
                        type="button"
                        class="value"
                        onclick={() => copy(value.data)}
                        title={$_('entry.copy_title')}
                      >
                        {value.data}
                      </button>
                    {/if}
                  </td>
                </tr>
              {/each}
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: 100%;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-wrap: wrap;
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.3rem;
    color: var(--color-text-muted);
  }

  .star {
    color: var(--color-text-subtle);
  }

  .star.on {
    color: var(--color-warning);
    background: color-mix(in oklab, var(--color-warning) 15%, transparent);
  }

  .dn {
    font-size: 0.82rem;
    color: var(--color-text);
    background: transparent;
    border: none;
    padding: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .pill {
    padding: 0.1rem 0.55rem;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .pill.dirty {
    background: color-mix(in oklab, var(--color-warning) 18%, transparent);
    color: var(--color-warning);
  }

  .search-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-wrap :global(svg) {
    position: absolute;
    left: 0.55rem;
    color: var(--color-text-subtle);
    pointer-events: none;
  }

  .search-wrap input {
    padding-left: 1.85rem;
    width: 12rem;
  }

  .status.banner {
    margin: 0.6rem 1rem 0;
    padding: 0.55rem 0.85rem;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  .status.error {
    color: var(--color-danger);
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 0.25rem 0.75rem 2rem;
  }

  /* -------- view mode table -------- */

  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    font-size: var(--text-sm);
  }

  thead th {
    text-align: left;
    padding: 0.55rem 0.75rem;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-subtle);
    border-bottom: 1px solid var(--color-border);
    position: sticky;
    top: 0;
    background: var(--color-bg);
    z-index: 1;
  }

  tbody th {
    text-align: left;
    vertical-align: top;
    padding: 0.45rem 0.75rem;
    width: 12rem;
    font-weight: 500;
    color: var(--color-text);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  tbody th code {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 0.78rem;
    color: var(--color-text);
  }

  tbody td {
    padding: 0.45rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text);
    vertical-align: top;
  }

  tbody tr:hover td,
  tbody tr:hover th {
    background: var(--color-surface-hover);
  }

  .value {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: copy;
    word-break: break-word;
  }

  .value:hover {
    text-decoration: underline dotted;
  }

  .binary {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-style: italic;
  }

  /* -------- edit mode -------- */

  .edit-area {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding-top: 0.75rem;
  }

  .edit-field {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }

  .edit-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.75rem;
    background: var(--color-surface-2);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .attr-name {
    font-family: var(--font-mono);
    font-size: 0.82rem;
    background: transparent;
    border: none;
    padding: 0;
    color: var(--color-text);
    font-weight: 500;
  }

  .value-count {
    font-size: 0.7rem;
    padding: 0.05rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
  }

  .edit-values {
    padding: 0.5rem 0.65rem;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .edit-row {
    display: flex;
    gap: 0.3rem;
  }

  .edit-row input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .add-val {
    align-self: flex-start;
  }

  .add-attr {
    display: flex;
    gap: 0.4rem;
    padding: 0.75rem;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface-2);
  }

  .add-attr input {
    flex: 1;
  }

  /* -------- notes -------- */

  .notes {
    margin: 0.25rem 0 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
  }

  .notes.open {
    background: var(--color-surface-2);
  }

  .notes-head {
    width: 100%;
    justify-content: flex-start;
    padding: 0.3rem 0.5rem;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.72rem;
    border-radius: var(--radius-md);
  }

  .notes-head .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--color-warning);
    margin-left: 0.2rem;
  }

  .notes-body {
    border: none;
    border-top: 1px solid var(--color-border);
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    padding: 0.5rem 0.6rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    resize: vertical;
    background: transparent;
    min-height: 3.2rem;
  }

  /* -------- empty state -------- */

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .empty-icon {
    display: grid;
    place-items: center;
    width: 3.2rem;
    height: 3.2rem;
    border-radius: var(--radius-xl);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text-subtle);
  }
</style>
