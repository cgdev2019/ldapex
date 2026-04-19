<script lang="ts">
  import { ping, type PingResponse } from '$lib/bridge';

  let result = $state<PingResponse | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function onPing() {
    loading = true;
    error = null;
    try {
      result = await ping();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }
</script>

<main>
  <h1>Ldapex</h1>
  <p>Application de bureau pour visualiser et modifier un annuaire LDAP.</p>

  <button onclick={onPing} disabled={loading}>
    {loading ? 'Ping…' : 'Tester le bridge Rust'}
  </button>

  {#if result}
    <dl class="result">
      <dt>message</dt>
      <dd><code>{result.message}</code></dd>
      <dt>core version</dt>
      <dd><code>{result.core_version}</code></dd>
      <dt>app version</dt>
      <dd><code>{result.app_version}</code></dd>
    </dl>
  {/if}

  {#if error}
    <p class="error">Erreur : {error}</p>
  {/if}
</main>

<style>
  main {
    max-width: 42rem;
    margin: 0 auto;
    padding: 2rem 1.5rem;
  }

  h1 {
    margin-top: 0;
  }

  .result {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.25rem 1rem;
    margin-top: 1.5rem;
  }

  .result dt {
    color: light-dark(#555, #aaa);
  }

  .result dd {
    margin: 0;
  }

  .error {
    color: #c0392b;
  }
</style>
