<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { formatError, type TlsMode } from '$lib/bridge';
  import { session } from '$lib/session.svelte';

  let url = $state('ldap://127.0.0.1:3389');
  let bindDn = $state('cn=admin,dc=ldapex,dc=test');
  let password = $state('');
  let baseDn = $state('dc=ldapex,dc=test');
  let tls = $state<TlsMode>('none');
  let error = $state<string | null>(null);

  async function onSubmit(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await session.connect({ url, bind_dn: bindDn, password, tls }, baseDn);
    } catch (err) {
      error = formatError(err);
    }
  }
</script>

<section class="card">
  <header>
    <h2>{$_('login.title')}</h2>
    <p class="hint">{$_('login.hint')}</p>
  </header>

  <form onsubmit={onSubmit}>
    <div class="grid">
      <label>
        <span>{$_('login.url')}</span>
        <input type="text" bind:value={url} required placeholder="ldap://host:389" />
      </label>

      <label>
        <span>{$_('login.tls')}</span>
        <select bind:value={tls}>
          <option value="none">{$_('login.tls_none')}</option>
          <option value="start_tls">{$_('login.tls_starttls')}</option>
          <option value="ldaps">{$_('login.tls_ldaps')}</option>
        </select>
      </label>

      <label class="span-2">
        <span>{$_('login.bind_dn')}</span>
        <input type="text" bind:value={bindDn} placeholder="cn=admin,dc=example,dc=org" />
      </label>

      <label class="span-2">
        <span>{$_('login.password')}</span>
        <input type="password" bind:value={password} autocomplete="current-password" />
      </label>

      <label class="span-2">
        <span>{$_('login.base_dn')}</span>
        <input type="text" bind:value={baseDn} required placeholder="dc=example,dc=org" />
      </label>
    </div>

    {#if error}
      <p class="status error">{error}</p>
    {/if}

    <button type="submit" class="primary lg" disabled={session.connecting}>
      {session.connecting ? $_('common.connecting') : $_('login.submit')}
    </button>
  </form>
</section>

<style>
  .card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-md);
    overflow: hidden;
  }

  header {
    padding: 1.1rem 1.5rem 0.6rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface);
  }

  header h2 {
    font-size: 1.05rem;
  }

  .hint {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    margin-top: 0.2rem;
  }

  form {
    padding: 1.1rem 1.5rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 11rem;
    gap: 0.9rem 1rem;
  }

  .span-2 {
    grid-column: span 2;
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

  :global(button.lg) {
    padding: 0.6rem 1.1rem;
    font-size: var(--text-base);
    font-weight: 600;
    align-self: flex-end;
    min-width: 9rem;
    justify-content: center;
  }

  .status.error {
    color: var(--color-danger);
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
    padding: 0.5rem 0.7rem;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }
</style>
