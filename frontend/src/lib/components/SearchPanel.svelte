<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import { formatError, ldapSearch, type Entry, type SearchScope } from '$lib/bridge';
  import { session } from '$lib/session.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    baseDn: string;
    onselect: (dn: string) => void;
  }

  let { baseDn, onselect }: Props = $props();

  let searchBase = $state(baseDn);
  let scope = $state<SearchScope>('subtree');
  let filter = $state('(objectClass=*)');
  let sizeLimit = $state<number>(100);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let results = $state<Entry[]>([]);
  let history = $state<string[]>([]);

  const HISTORY_MAX = 20;

  $effect(() => {
    searchBase = baseDn;
  });

  $effect(() => {
    history = loadHistory(historyKey());
  });

  function historyKey(): string {
    const id = session.activeProfileId ?? 'adhoc';
    return `ldapex.search-history.${id}`;
  }

  function loadHistory(key: string): string[] {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return [];
      const parsed: unknown = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed.filter((v): v is string => typeof v === 'string');
    } catch {
      return [];
    }
  }

  function rememberFilter(value: string) {
    const trimmed = value.trim();
    if (!trimmed) return;
    const key = historyKey();
    const next = [trimmed, ...history.filter((h) => h !== trimmed)].slice(0, HISTORY_MAX);
    history = next;
    try {
      localStorage.setItem(key, JSON.stringify(next));
    } catch {
      /* ignore */
    }
  }

  function clearHistory() {
    if (history.length === 0) return;
    if (!window.confirm(get(_)('search.clear_history_confirm'))) return;
    history = [];
    try {
      localStorage.removeItem(historyKey());
    } catch {
      /* ignore */
    }
  }

  async function run(event: SubmitEvent) {
    event.preventDefault();
    loading = true;
    error = null;
    try {
      results = await ldapSearch({
        base_dn: searchBase,
        scope,
        filter,
        attributes: ['cn', 'uid', 'ou', 'mail'],
        size_limit: sizeLimit > 0 ? sizeLimit : null
      });
      rememberFilter(filter);
    } catch (err) {
      error = formatError(err);
      results = [];
    } finally {
      loading = false;
    }
  }

  function labelOf(e: Entry): string {
    for (const prefer of ['cn', 'uid', 'ou']) {
      const attr = e.attributes.find((a) => a.name.toLowerCase() === prefer);
      const value = attr?.values.find((v) => v.kind === 'text');
      if (value && value.kind === 'text') return value.data;
    }
    return e.dn;
  }
</script>

<form onsubmit={run}>
  <label>
    <span>{$_('search.base_dn')}</span>
    <input type="text" bind:value={searchBase} required spellcheck="false" />
  </label>

  <div class="row">
    <label class="flex">
      <span>{$_('search.scope')}</span>
      <select bind:value={scope}>
        <option value="base">base</option>
        <option value="one_level">onelevel</option>
        <option value="subtree">subtree</option>
      </select>
    </label>

    <label class="sm">
      <span>{$_('search.size_limit')}</span>
      <input type="number" min="0" bind:value={sizeLimit} />
    </label>
  </div>

  <label>
    <span>{$_('search.filter')}</span>
    <input
      type="text"
      bind:value={filter}
      placeholder="(&amp;(objectClass=person)(cn=a*))"
      spellcheck="false"
      list="ldapex-search-history"
    />
    <datalist id="ldapex-search-history">
      {#each history as h (h)}
        <option value={h}></option>
      {/each}
    </datalist>
  </label>

  <div class="actions">
    <button type="submit" class="primary sm" disabled={loading}>
      <Icon name={loading ? 'refresh' : 'search'} size={13} />
      <span>{loading ? $_('search.searching') : $_('search.submit')}</span>
    </button>
    {#if history.length > 0}
      <button
        type="button"
        class="ghost sm"
        onclick={clearHistory}
        title={$_('search.clear_history_tooltip')}
      >
        <Icon name="x" size={12} />
        <span>{$_('search.clear_history', { values: { count: history.length } })}</span>
      </button>
    {/if}
  </div>
</form>

<div class="results-wrap">
  {#if error}
    <p class="status error">{error}</p>
  {:else if results.length > 0}
    <div class="results-head">
      <span>{results.length} {results.length === 1 ? 'result' : 'results'}</span>
    </div>
    <ul class="results">
      {#each results as entry (entry.dn)}
        <li>
          <button type="button" onclick={() => onselect(entry.dn)} title={entry.dn}>
            <Icon name="circle-dot" size={12} />
            <span class="texts">
              <span class="label">{labelOf(entry)}</span>
              <span class="dn">{entry.dn}</span>
            </span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if !loading}
    <p class="status muted">
      <Icon name="search" size={13} />
      <span>{$_('search.no_results')}</span>
    </p>
  {/if}
</div>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .row {
    display: flex;
    gap: 0.5rem;
  }

  .flex {
    flex: 1;
  }

  .sm {
    width: 5.5rem;
    flex-shrink: 0;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: var(--text-xs);
  }

  label > span {
    color: var(--color-text-muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.65rem;
  }

  input,
  select {
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }

  .actions {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .results-wrap {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .results-head {
    padding: 0.4rem 0.75rem;
    font-size: 0.65rem;
    color: var(--color-text-subtle);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface-2);
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.55rem 0.75rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .status.error {
    color: var(--color-danger);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .status.muted {
    font-style: italic;
    color: var(--color-text-subtle);
  }

  .results {
    list-style: none;
    margin: 0;
    padding: 0.25rem;
  }

  .results li {
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .results li:hover {
    background: var(--color-surface-hover);
  }

  .results button {
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 0.5rem;
    width: 100%;
    background: none;
    border: none;
    color: inherit;
    padding: 0.4rem 0.55rem;
    cursor: pointer;
    text-align: left;
    border-radius: var(--radius-md);
  }

  .results button:hover {
    background: transparent;
  }

  .results :global(svg) {
    color: var(--color-primary);
    margin-top: 0.2rem;
    flex-shrink: 0;
  }

  .texts {
    display: flex;
    flex-direction: column;
    gap: 0.05rem;
    min-width: 0;
    flex: 1;
  }

  .label {
    font-weight: 500;
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dn {
    font-size: 0.72rem;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
