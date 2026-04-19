<script lang="ts">
  import { formatError, ldapReadEntry, type Entry } from '$lib/bridge';

  interface Props {
    dn: string | null;
  }

  let { dn }: Props = $props();

  let entry = $state<Entry | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let filter = $state('');

  $effect(() => {
    if (dn) {
      load(dn);
    } else {
      entry = null;
      error = null;
    }
  });

  async function load(target: string) {
    loading = true;
    error = null;
    try {
      entry = await ldapReadEntry(target);
    } catch (err) {
      error = formatError(err);
      entry = null;
    } finally {
      loading = false;
    }
  }

  const filtered = $derived(
    entry && filter
      ? entry.attributes.filter((a) =>
          a.name.toLowerCase().includes(filter.toLowerCase())
        )
      : entry?.attributes ?? []
  );

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // Clipboard API may be unavailable; fail silently for MVP.
    }
  }
</script>

<section>
  {#if !dn}
    <p class="empty">Sélectionne une entrée dans l'arbre pour afficher ses attributs.</p>
  {:else if loading}
    <p class="status">Chargement…</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if entry}
    <header>
      <code class="dn" title={entry.dn}>{entry.dn}</code>
      <input
        type="search"
        placeholder="Filtrer les attributs…"
        bind:value={filter}
      />
    </header>

    <table>
      <thead>
        <tr>
          <th scope="col">Attribut</th>
          <th scope="col">Valeur</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as attr (attr.name)}
          {#each attr.values as value, i (i)}
            <tr>
              {#if i === 0}
                <th scope="row" rowspan={attr.values.length}>{attr.name}</th>
              {/if}
              <td>
                {#if value.kind === 'text'}
                  <button
                    type="button"
                    class="value"
                    onclick={() => copy(value.data)}
                    title="Copier"
                  >
                    {value.data}
                  </button>
                {:else}
                  <span class="binary" title="{value.data.length} caractères base64">
                    &lt;binaire — {value.data.length} b64&gt;
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        {/each}
      </tbody>
    </table>
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

  header input {
    font: inherit;
    padding: 0.35rem 0.55rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
    min-width: 14rem;
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
</style>
