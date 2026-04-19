<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    ldapDelete,
    ldapModify,
    ldapReadEntry,
    type Attribute,
    type AttributeValue,
    type Entry,
    type Modification
  } from '$lib/bridge';

  interface Props {
    dn: string | null;
    onentrychanged?: (event: { dn: string; kind: 'modified' | 'deleted' }) => void;
  }

  let { dn, onentrychanged }: Props = $props();

  let entry = $state<Entry | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let filter = $state('');

  // Edit mode keeps a working copy of the text attributes (binary ones
  // are kept read-only in MVP). Saving computes the per-attribute
  // diff against `originalText`.
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
    } catch (err) {
      error = formatError(err);
      entry = null;
    } finally {
      loading = false;
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

  async function save() {
    if (!dn) return;
    const mods = computeDiff();
    if (mods.length === 0) {
      editing = false;
      return;
    }
    saving = true;
    error = null;
    try {
      await ldapModify(dn, mods);
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
    try {
      await ldapDelete(dn);
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
      await navigator.clipboard.writeText(text);
    } catch {
      /* clipboard may be unavailable — ignore in MVP */
    }
  }

  const displayAttributes = $derived(
    entry && filter
      ? entry.attributes.filter((a) => a.name.toLowerCase().includes(filter.toLowerCase()))
      : (entry?.attributes ?? [])
  );

  function binaryValues(attr: Attribute): AttributeValue[] {
    return attr.values.filter((v) => v.kind === 'binary');
  }
</script>

<section>
  {#if !dn}
    <p class="empty">{$_('entry.empty')}</p>
  {:else if loading}
    <p class="status">{$_('common.loading')}</p>
  {:else if !entry}
    {#if error}
      <p class="status error">{error}</p>
    {/if}
  {:else}
    <header>
      <code class="dn" title={entry.dn}>{entry.dn}</code>
      <div class="actions">
        {#if editing}
          <button type="button" onclick={save} disabled={saving}>
            {saving ? $_('entry.actions.saving') : $_('entry.actions.save')}
          </button>
          <button type="button" class="secondary" onclick={cancelEdit} disabled={saving}>
            {$_('entry.actions.cancel')}
          </button>
        {:else}
          <input
            type="search"
            placeholder={$_('entry.filter_placeholder')}
            bind:value={filter}
          />
          <button type="button" onclick={() => (editing = true)}>{$_('entry.actions.edit')}</button>
          <button type="button" class="danger" onclick={confirmDelete} disabled={saving}>
            {$_('entry.actions.delete')}
          </button>
        {/if}
      </div>
    </header>

    {#if error}
      <p class="status error">{error}</p>
    {/if}

    {#if editing}
      <div class="edit-area">
        {#each Object.entries(draftText) as [name, values] (name)}
          <fieldset>
            <legend>{name}</legend>
            {#each values as _value, i}
              <div class="row">
                <input
                  type="text"
                  value={values[i]}
                  oninput={(e) =>
                    updateValue(name, i, (e.currentTarget as HTMLInputElement).value)}
                />
                <button type="button" class="icon-btn" onclick={() => removeValue(name, i)}>
                  ×
                </button>
              </div>
            {/each}
            <button type="button" class="tertiary" onclick={() => addValue(name)}>
              {$_('entry.add_value')}
            </button>
          </fieldset>
        {/each}

        <div class="add-attr">
          <input
            type="text"
            placeholder={$_('entry.new_attribute_placeholder')}
            bind:value={newAttrName}
            onkeydown={(e) => e.key === 'Enter' && addAttribute()}
          />
          <button type="button" onclick={addAttribute}>{$_('entry.add_attribute')}</button>
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
                  <th scope="row" rowspan={values.length}>{attr.name}</th>
                {/if}
                <td>
                  {#if value.kind === 'text'}
                    <button
                      type="button"
                      class="value"
                      onclick={() => copy(value.data)}
                      title={$_('entry.copy_title')}
                    >
                      {value.data}
                    </button>
                  {:else}
                    <span class="binary" title="{value.data.length} base64 chars">
                      {$_('entry.binary_value', { values: { count: value.data.length } })}
                    </span>
                  {/if}
                </td>
              </tr>
            {/each}
            {#if binaryValues(attr).length > 0 && values.every((v) => v.kind === 'binary')}
              <!-- already rendered in loop -->
            {/if}
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  section {
    overflow: auto;
    padding: 0.75rem 1rem;
  }

  .empty,
  .status {
    color: light-dark(#666, #888);
    margin: 0.5rem 0;
  }

  .status.error {
    color: #c0392b;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }

  .dn {
    word-break: break-all;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  header input[type='search'],
  .add-attr input,
  fieldset input[type='text'] {
    font: inherit;
    padding: 0.35rem 0.55rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
  }

  header input[type='search'] {
    min-width: 12rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  th,
  td {
    text-align: left;
    vertical-align: top;
    padding: 0.3rem 0.5rem;
    border-bottom: 1px solid light-dark(#eee, #262626);
  }

  th {
    width: 14rem;
    font-weight: 600;
    color: light-dark(#333, #ddd);
  }

  .value {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: copy;
    text-align: left;
    word-break: break-word;
  }

  .value:hover {
    text-decoration: underline dotted;
  }

  .binary {
    font-style: italic;
    color: light-dark(#777, #888);
  }

  .edit-area {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  fieldset {
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 5px;
    padding: 0.5rem 0.75rem;
  }

  fieldset legend {
    font-weight: 600;
    padding: 0 0.25rem;
    font-size: 0.9rem;
  }

  .row {
    display: flex;
    gap: 0.35rem;
    margin-bottom: 0.3rem;
  }

  .row input {
    flex: 1;
  }

  .icon-btn {
    width: 1.75rem;
    padding: 0;
  }

  .add-attr {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .add-attr input {
    flex: 1;
  }

  .danger {
    border-color: #c0392b;
    color: #c0392b;
  }

  .danger:hover {
    background: #fdedec;
  }

  .secondary,
  .tertiary {
    background: transparent;
  }

  .tertiary {
    border: none;
    color: light-dark(#0057b7, #7aaeff);
    padding: 0.1rem 0.25rem;
    font-size: 0.85rem;
  }

  .tertiary:hover {
    text-decoration: underline;
  }
</style>
