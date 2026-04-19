<script lang="ts">
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
      await session.connect(
        { url, bind_dn: bindDn, password, tls },
        baseDn
      );
    } catch (err) {
      error = formatError(err);
    }
  }
</script>

<form onsubmit={onSubmit}>
  <h1>Connexion LDAP</h1>

  <label>
    <span>URL</span>
    <input type="text" bind:value={url} required placeholder="ldap://host:389" />
  </label>

  <label>
    <span>Bind DN</span>
    <input type="text" bind:value={bindDn} placeholder="cn=admin,dc=example,dc=org" />
  </label>

  <label>
    <span>Mot de passe</span>
    <input type="password" bind:value={password} autocomplete="current-password" />
  </label>

  <label>
    <span>Base DN</span>
    <input type="text" bind:value={baseDn} required placeholder="dc=example,dc=org" />
  </label>

  <label>
    <span>TLS</span>
    <select bind:value={tls}>
      <option value="none">Aucun (ldap://)</option>
      <option value="start_tls">StartTLS</option>
      <option value="ldaps">LDAPS (ldaps://)</option>
    </select>
  </label>

  <button type="submit" disabled={session.connecting}>
    {session.connecting ? 'Connexion…' : 'Se connecter'}
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}
</form>

<style>
  form {
    max-width: 28rem;
    margin: 2rem auto;
    padding: 1.5rem;
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    background: light-dark(#fff, #161616);
  }

  h1 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.9rem;
  }

  label > span {
    color: light-dark(#555, #aaa);
  }

  input,
  select {
    font: inherit;
    padding: 0.45rem 0.6rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
  }

  button {
    margin-top: 0.25rem;
  }

  .error {
    color: #c0392b;
    margin: 0;
    font-size: 0.9rem;
  }
</style>
