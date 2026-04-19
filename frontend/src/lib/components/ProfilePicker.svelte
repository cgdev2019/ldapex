<script lang="ts">
  import {
    formatError,
    profileDelete,
    profileExport,
    profileImport,
    profileList,
    type ProfileSummary
  } from '$lib/bridge';
  import { session } from '$lib/session.svelte';
  import ProfileEditor from './ProfileEditor.svelte';

  let profiles = $state<ProfileSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editing = $state<ProfileSummary | null>(null);
  let creating = $state(false);

  let passwordPromptFor = $state<ProfileSummary | null>(null);
  let pendingPassword = $state('');
  let pendingRemember = $state(false);

  $effect(() => {
    reload();
  });

  async function reload() {
    loading = true;
    error = null;
    try {
      profiles = await profileList();
    } catch (err) {
      error = formatError(err);
    } finally {
      loading = false;
    }
  }

  async function connect(profile: ProfileSummary) {
    if (!profile.has_saved_password) {
      passwordPromptFor = profile;
      pendingPassword = '';
      pendingRemember = profile.save_password;
      return;
    }
    try {
      await session.connectWithProfile({ id: profile.id });
    } catch (err) {
      error = formatError(err);
    }
  }

  async function submitPassword(event: SubmitEvent) {
    event.preventDefault();
    if (!passwordPromptFor) return;
    const target = passwordPromptFor;
    try {
      await session.connectWithProfile({
        id: target.id,
        password: pendingPassword,
        remember: pendingRemember
      });
      passwordPromptFor = null;
      pendingPassword = '';
    } catch (err) {
      error = formatError(err);
    }
  }

  async function confirmDelete(profile: ProfileSummary) {
    const yes = window.confirm(
      `Supprimer le profil « ${profile.name} » ?\n\nLe mot de passe stocké sera aussi retiré du trousseau.`
    );
    if (!yes) return;
    try {
      await profileDelete(profile.id);
      await reload();
    } catch (err) {
      error = formatError(err);
    }
  }

  async function doExport() {
    try {
      const json = await profileExport();
      await navigator.clipboard.writeText(json);
      window.alert('Profils copiés dans le presse-papier (JSON).');
    } catch (err) {
      error = formatError(err);
    }
  }

  async function doImport() {
    const raw = window.prompt('Colle ici un export JSON de profils :');
    if (!raw) return;
    try {
      await profileImport(raw);
      await reload();
    } catch (err) {
      error = formatError(err);
    }
  }
</script>

<section>
  <header>
    <h2>Profils enregistrés</h2>
    <div class="tools">
      <button type="button" class="tertiary" onclick={doImport}>Importer…</button>
      <button type="button" class="tertiary" onclick={doExport} disabled={profiles.length === 0}>
        Exporter
      </button>
      <button type="button" onclick={() => (creating = true)}>+ Nouveau</button>
    </div>
  </header>

  {#if loading}
    <p class="status">Chargement…</p>
  {:else if error}
    <p class="status error">{error}</p>
  {/if}

  {#if profiles.length > 0}
    <ul>
      {#each profiles as p (p.id)}
        <li>
          <button type="button" class="row" onclick={() => connect(p)} disabled={session.connecting}>
            <div class="line1">
              <strong>{p.name}</strong>
              {#if p.has_saved_password}
                <span class="tag" title="Mot de passe en trousseau OS">🔑</span>
              {/if}
              <span class="tls">{p.tls}</span>
            </div>
            <div class="line2">
              {p.bind_dn || '(anonyme)'} @ {p.url}
            </div>
            <div class="line3">{p.base_dn}</div>
          </button>
          <div class="actions">
            <button type="button" class="icon-btn" onclick={() => (editing = p)} title="Modifier">
              ✎
            </button>
            <button
              type="button"
              class="icon-btn danger"
              onclick={() => confirmDelete(p)}
              title="Supprimer"
            >
              ×
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {:else if !loading}
    <p class="empty">Aucun profil. Clique sur « + Nouveau » pour en créer un.</p>
  {/if}
</section>

{#if creating}
  <ProfileEditor
    onclose={() => (creating = false)}
    onsaved={async () => {
      creating = false;
      await reload();
    }}
  />
{/if}

{#if editing}
  <ProfileEditor
    profile={editing}
    onclose={() => (editing = null)}
    onsaved={async () => {
      editing = null;
      await reload();
    }}
  />
{/if}

{#if passwordPromptFor}
  <div class="backdrop" role="dialog" aria-modal="true" aria-label="Mot de passe">
    <form class="pw-dialog" onsubmit={submitPassword}>
      <h3>Mot de passe pour « {passwordPromptFor.name} »</h3>
      <p class="bind">{passwordPromptFor.bind_dn || '(anonyme)'}</p>

      <input
        type="password"
        bind:value={pendingPassword}
        autocomplete="current-password"
        autofocus
        required
      />

      <label class="inline">
        <input type="checkbox" bind:checked={pendingRemember} />
        <span>Mémoriser dans le trousseau OS</span>
      </label>

      <div class="actions">
        <button type="submit" disabled={session.connecting}>
          {session.connecting ? 'Connexion…' : 'Se connecter'}
        </button>
        <button
          type="button"
          class="secondary"
          onclick={() => (passwordPromptFor = null)}
          disabled={session.connecting}
        >
          Annuler
        </button>
      </div>
    </form>
  </div>
{/if}

<style>
  section {
    max-width: 42rem;
    margin: 1.5rem auto;
    padding: 1rem 1.5rem;
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 8px;
    background: light-dark(#fff, #161616);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  header h2 {
    margin: 0;
    font-size: 1.05rem;
  }

  .tools {
    display: flex;
    gap: 0.5rem;
  }

  .tertiary {
    background: transparent;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    border: 1px solid light-dark(#eee, #262626);
    border-radius: 5px;
    padding: 0.35rem 0.5rem;
  }

  .row {
    flex: 1;
    text-align: left;
    background: none;
    border: none;
    padding: 0.3rem;
    color: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .row:hover {
    background: light-dark(#f4f4f4, #1a1a1a);
  }

  .row[disabled] {
    cursor: default;
    opacity: 0.6;
  }

  .line1 {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.95rem;
  }

  .line2,
  .line3 {
    font-size: 0.8rem;
    color: light-dark(#666, #888);
    word-break: break-all;
  }

  .tag {
    font-size: 0.9rem;
  }

  .tls {
    padding: 0 0.3rem;
    border-radius: 3px;
    background: light-dark(#eee, #222);
    color: light-dark(#444, #bbb);
    font-size: 0.7rem;
    text-transform: uppercase;
  }

  .empty,
  .status {
    color: light-dark(#666, #888);
  }

  .status.error {
    color: #c0392b;
  }

  .icon-btn {
    width: 1.8rem;
    padding: 0.25rem;
    background: transparent;
  }

  .danger {
    color: #c0392b;
    border-color: #c0392b;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 110;
  }

  .pw-dialog {
    background: light-dark(#fff, #161616);
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    width: min(90vw, 24rem);
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .pw-dialog h3 {
    margin: 0;
    font-size: 1rem;
  }

  .bind {
    font-size: 0.8rem;
    color: light-dark(#666, #888);
    margin: 0;
  }

  input[type='password'] {
    font: inherit;
    padding: 0.4rem 0.55rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
  }

  .inline {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .secondary {
    background: transparent;
  }
</style>
