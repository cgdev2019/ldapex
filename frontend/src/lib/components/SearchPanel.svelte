<script lang="ts">
  import { formatError, ldapSearch, type Entry, type SearchScope } from '$lib/bridge';

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

  $effect(() => {
    // If the caller changes the base DN (e.g. fresh login), prefill it
    // but leave the filter/scope alone.
    searchBase = baseDn;
  });

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
    <span>Base DN</span>
    <input type="text" bind:value={searchBase} required />
  </label>

  <div class="row">
    <label>
      <span>Scope</span>
      <select bind:value={scope}>
        <option value="base">base</option>
        <option value="one_level">onelevel</option>
        <option value="subtree">subtree</option>
      </select>
    </label>

    <label>
      <span>Limite</span>
      <input type="number" min="0" bind:value={sizeLimit} />
    </label>
  </div>

  <label>
    <span>Filtre (RFC 4515)</span>
    <input
      type="text"
      bind:value={filter}
      placeholder="(&amp;(objectClass=person)(cn=a*))"
      spellcheck="false"
    />
  </label>

  <button type="submit" disabled={loading}>
    {loading ? 'Recherche…' : 'Rechercher'}
  </button>
</form>

{#if error}
  <p class="status error">{error}</p>
{:else if results.length > 0}
  <ul class="results">
    {#each results as entry (entry.dn)}
      <li>
        <button type="button" onclick={() => onselect(entry.dn)} title={entry.dn}>
          <span class="label">{labelOf(entry)}</span>
          <span class="dn">{entry.dn}</span>
        </button>
      </li>
    {/each}
  </ul>
{:else if !loading}
  <p class="status muted">Aucun résultat (lance une recherche).</p>
{/if}

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem;
  }

  .row {
    display: flex;
    gap: 0.5rem;
  }

  .row label {
    flex: 1;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.85rem;
  }

  label > span {
    color: light-dark(#555, #aaa);
  }

  input,
  select {
    font: inherit;
    padding: 0.35rem 0.5rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
  }

  .status {
    padding: 0.5rem;
    color: light-dark(#666, #888);
    margin: 0;
  }

  .status.error {
    color: #c0392b;
  }

  .status.muted {
    font-size: 0.85rem;
    font-style: italic;
  }

  .results {
    list-style: none;
    margin: 0;
    padding: 0 0.25rem;
    overflow: auto;
  }

  .results li {
    border-bottom: 1px solid light-dark(#eee, #262626);
  }

  .results button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.1rem;
    width: 100%;
    background: none;
    border: none;
    color: inherit;
    padding: 0.4rem 0.4rem;
    cursor: pointer;
    text-align: left;
  }

  .results button:hover {
    background: light-dark(#f3f3f3, #1c1c1c);
  }

  .label {
    font-weight: 500;
  }

  .dn {
    font-size: 0.8rem;
    color: light-dark(#666, #888);
    word-break: break-all;
  }
</style>
